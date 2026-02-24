use crate::callrecord::{CallRecord, CallRecordHook};
use crate::config::TranscriptToolConfig;
use crate::console::handlers::utils::{build_sensevoice_transcribe_command, command_exists};
use crate::models::call_record::{
    ActiveModel as CallRecordActiveModel, Column as CallRecordColumn, Entity as CallRecordEntity,
};
use anyhow::Result;
use chrono::Utc;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn};

use super::models::{
    ChannelParticipant, SenseVoiceCliChannel, StoredTranscript, StoredTranscriptAnalysis,
    StoredTranscriptSegment,
};
use std::collections::HashMap;

/// A `CallRecordHook` that automatically transcribes recordings after a call
/// record is saved. Runs the configured transcription command (e.g.
/// `groq-sensevoice-wrapper` or `sensevoice-cli`) against the WAV file and
/// writes a `.transcript.json` sidecar next to the recording.
pub struct TranscriptHook {
    db: DatabaseConnection,
    config: TranscriptToolConfig,
}

impl TranscriptHook {
    pub fn new(db: DatabaseConnection, config: TranscriptToolConfig) -> Self {
        Self { db, config }
    }

    /// Resolve the transcript command, defaulting to "sensevoice-cli".
    fn command(&self) -> String {
        self.config
            .command
            .as_deref()
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .unwrap_or("sensevoice-cli")
            .to_string()
    }

    /// Resolve the models path from config or environment.
    fn models_path(&self) -> Option<String> {
        if let Ok(env_path) = std::env::var("MODEL_PATH") {
            let trimmed = env_path.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
        self.config
            .models_path
            .as_ref()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string())
    }

    /// Pick the first recording file path that exists on disk.
    fn find_recording_path(record: &CallRecord) -> Option<String> {
        // Check recorder entries first
        for media in &record.recorder {
            let path = media.path.trim();
            if !path.is_empty() && Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
        // Fallback to details.recording_url
        if let Some(url) = record.details.recording_url.as_ref() {
            let trimmed = url.trim();
            if !trimmed.is_empty() && Path::new(trimmed).exists() {
                return Some(trimmed.to_string());
            }
        }
        None
    }

    /// Compute the transcript output path as a sidecar to the recording file.
    fn transcript_output_path(recording_path: &str) -> PathBuf {
        let mut path = PathBuf::from(recording_path);
        // Replace .wav with .transcript.json
        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        path.set_file_name(format!("{}.transcript.json", stem));
        path
    }

    /// Run the transcription command and return the parsed transcript.
    /// The `record` is used to label channels with caller/callee roles.
    async fn run_transcription(
        &self,
        recording_path: &str,
        output_path: &Path,
        record: &CallRecord,
    ) -> Result<StoredTranscript> {
        let command = self.command();
        let models_path = self.models_path();
        let output_str = output_path.to_string_lossy();

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut cmd = build_sensevoice_transcribe_command(
            &command,
            recording_path,
            models_path.as_deref(),
            Some(&output_str),
        );

        let start_instant = std::time::Instant::now();
        let timeout_secs = self.config.timeout_secs.unwrap_or(120);

        let output_result = match timeout(Duration::from_secs(timeout_secs), cmd.output()).await {
            Ok(result) => result,
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "transcription command timed out after {} seconds",
                    timeout_secs
                ));
            }
        };

        let output = output_result?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "transcription command exited with status {}: {}",
                output.status.code().unwrap_or(-1),
                stderr.chars().take(200).collect::<String>()
            ));
        }

        let elapsed_secs = start_instant.elapsed().as_secs_f64();

        // Read and parse the output file
        let transcript_bytes = tokio::fs::read(output_path).await.map_err(|e| {
            anyhow::anyhow!(
                "transcription command succeeded but output file missing at {}: {}",
                output_path.display(),
                e
            )
        })?;

        let cli_output: Vec<SenseVoiceCliChannel> = serde_json::from_slice(&transcript_bytes)?;

        // Build channel-to-participant mapping.
        // Stereo WAV recorder: channel 0 = leg A = caller, channel 1 = leg B = callee.
        let caller_label = record
            .details
            .from_number
            .as_deref()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                let c = record.caller.trim();
                if c.is_empty() { None } else { Some(c) }
            })
            .map(|s| s.to_string());

        let callee_label = record
            .details
            .to_number
            .as_deref()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                let c = record.callee.trim();
                if c.is_empty() { None } else { Some(c) }
            })
            .map(|s| s.to_string());

        let mut channel_labels = HashMap::new();
        channel_labels.insert(
            "0".to_string(),
            ChannelParticipant {
                role: "caller".to_string(),
                label: caller_label.clone(),
            },
        );
        channel_labels.insert(
            "1".to_string(),
            ChannelParticipant {
                role: "callee".to_string(),
                label: callee_label.clone(),
            },
        );

        // Convert CLI output to StoredTranscript
        let mut segments = Vec::new();
        let mut full_text = String::new();
        let mut total_word_count = 0;

        for channel in cli_output {
            let ch_num = channel.channel.unwrap_or(0);
            let (role, label) = match ch_num {
                0 => ("caller".to_string(), caller_label.clone()),
                1 => ("callee".to_string(), callee_label.clone()),
                n => (format!("channel_{}", n), None),
            };

            for segment in channel.segments {
                let text = segment.text.trim();
                if !text.is_empty() {
                    if !full_text.is_empty() {
                        full_text.push(' ');
                    }
                    full_text.push_str(text);
                    total_word_count += text.split_whitespace().count();
                }
                segments.push(StoredTranscriptSegment {
                    idx: None,
                    text: text.to_string(),
                    start: segment.start_sec,
                    end: segment.end_sec,
                    channel: channel.channel,
                    role: Some(role.clone()),
                    label: label.clone(),
                });
            }
        }

        let stored = StoredTranscript {
            version: 1,
            source: command.clone(),
            generated_at: Utc::now(),
            language: None,
            duration_secs: Some(elapsed_secs),
            sample_rate: None,
            segments,
            text: full_text,
            analysis: Some(StoredTranscriptAnalysis {
                elapsed: Some(elapsed_secs),
                rtf: None,
                word_count: total_word_count,
                asr_model: Some(command),
            }),
            channel_labels,
        };

        // Write the enriched transcript back (overwrite the raw CLI output)
        let json_content = serde_json::to_string_pretty(&stored)?;
        tokio::fs::write(output_path, &json_content).await?;

        Ok(stored)
    }

    /// Update the database call record with transcript status.
    async fn update_db_status(
        &self,
        call_id: &str,
        has_transcript: bool,
        status: &str,
    ) -> Result<()> {
        let now = Utc::now();
        // Find the record by call_id
        let record = CallRecordEntity::find()
            .filter(CallRecordColumn::CallId.eq(call_id))
            .one(&self.db)
            .await?;

        if let Some(record) = record {
            let active = CallRecordActiveModel {
                id: Set(record.id),
                has_transcript: Set(has_transcript),
                transcript_status: Set(status.to_string()),
                updated_at: Set(now),
                ..Default::default()
            };
            use sea_orm::ActiveModelTrait;
            active.update(&self.db).await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl CallRecordHook for TranscriptHook {
    async fn on_record_completed(&self, record: &mut CallRecord) -> Result<()> {
        // Only transcribe if there is a recording file
        let recording_path = match Self::find_recording_path(record) {
            Some(path) => path,
            None => {
                tracing::debug!(
                    call_id = %record.call_id,
                    "no recording file found, skipping auto-transcription"
                );
                return Ok(());
            }
        };

        let command = self.command();
        if !command_exists(&command) {
            warn!(
                call_id = %record.call_id,
                command = %command,
                "transcription command not found, skipping auto-transcription"
            );
            return Ok(());
        }

        let output_path = Self::transcript_output_path(&recording_path);

        info!(
            call_id = %record.call_id,
            recording = %recording_path,
            output = %output_path.display(),
            command = %command,
            "starting auto-transcription"
        );

        // Mark as processing in the database
        if let Err(e) = self
            .update_db_status(&record.call_id, false, "processing")
            .await
        {
            warn!(
                call_id = %record.call_id,
                "failed to update transcript status to processing: {}", e
            );
        }

        match self.run_transcription(&recording_path, &output_path, record).await {
            Ok(transcript) => {
                info!(
                    call_id = %record.call_id,
                    segments = transcript.segments.len(),
                    text_len = transcript.text.len(),
                    elapsed = ?transcript.duration_secs,
                    "auto-transcription completed successfully"
                );

                // Update the CallRecord in-memory so downstream hooks see the result
                record.details.has_transcript = true;
                record.details.transcript_status = Some("completed".to_string());

                // Update database
                if let Err(e) = self
                    .update_db_status(&record.call_id, true, "completed")
                    .await
                {
                    warn!(
                        call_id = %record.call_id,
                        "failed to update transcript status to completed: {}", e
                    );
                }
            }
            Err(e) => {
                warn!(
                    call_id = %record.call_id,
                    recording = %recording_path,
                    "auto-transcription failed: {}", e
                );

                record.details.transcript_status = Some("failed".to_string());

                if let Err(db_err) = self
                    .update_db_status(&record.call_id, false, "failed")
                    .await
                {
                    warn!(
                        call_id = %record.call_id,
                        "failed to update transcript status to failed: {}", db_err
                    );
                }
            }
        }

        Ok(())
    }
}
