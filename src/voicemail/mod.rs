use crate::config::VoicemailConfig;
use crate::media::recorder::RecorderOption;
use crate::models::voicemail;
use anyhow::{Result, anyhow};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection};
use std::path::PathBuf;
use tracing::info;

/// VoicemailService manages voicemail deposit operations.
///
/// It coordinates greeting playback, message recording, database
/// persistence, and optional post-recording transcription.
pub struct VoicemailService {
    config: VoicemailConfig,
    db: Option<DatabaseConnection>,
}

/// Information about a deposited voicemail message.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VoicemailDepositResult {
    pub recording_path: String,
    pub duration_secs: i32,
    pub mailbox_id: String,
}

impl VoicemailService {
    /// Create a new VoicemailService with the given configuration.
    pub fn new(config: VoicemailConfig, db: Option<DatabaseConnection>) -> Self {
        Self { config, db }
    }

    /// Check whether voicemail is globally enabled.
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Return the maximum message duration in seconds.
    #[allow(dead_code)]
    pub fn max_duration_secs(&self) -> u64 {
        self.config.max_message_duration_secs
    }

    /// Build a recorder option for a voicemail deposit.
    pub fn build_recorder_option(
        &self,
        call_id: &str,
        mailbox_id: &str,
    ) -> RecorderOption {
        let storage_path = &self.config.storage_path;
        let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let filename = format!("vm-{}-{}-{}.wav", mailbox_id, timestamp, call_id);
        let mut path = PathBuf::from(storage_path);
        path.push(mailbox_id);
        // Ensure the directory exists
        std::fs::create_dir_all(&path).ok();
        path.push(&filename);

        RecorderOption {
            recorder_file: path.to_string_lossy().to_string(),
            samplerate: 8000,
            ptime: 200,
            input_gain: 1.0,
            output_gain: 1.0,
        }
    }

    /// Resolve the greeting audio file path for a given mailbox.
    ///
    /// Looks for a custom greeting first, then falls back to a default.
    /// Returns None if no greeting file exists.
    pub fn resolve_greeting_path(&self, mailbox_id: &str) -> Option<String> {
        // Try custom greeting for this mailbox
        let custom = PathBuf::from(&self.config.greeting_path)
            .join(format!("{}.wav", mailbox_id));
        if custom.exists() {
            return Some(custom.to_string_lossy().to_string());
        }

        // Try default greeting
        let default_greeting = PathBuf::from(&self.config.greeting_path)
            .join("default.wav");
        if default_greeting.exists() {
            return Some(default_greeting.to_string_lossy().to_string());
        }

        None
    }

    /// Save a voicemail record to the database after recording completes.
    pub async fn save_voicemail_record(
        &self,
        call_id: &str,
        mailbox_id: &str,
        caller_id: &str,
        caller_name: Option<String>,
        recording_path: &str,
        duration_secs: i32,
    ) -> Result<()> {
        let db = match &self.db {
            Some(db) => db,
            None => {
                info!(
                    mailbox_id,
                    recording_path,
                    duration_secs,
                    "Voicemail recorded (no database configured, skipping DB save)"
                );
                return Ok(());
            }
        };

        let now = Utc::now();
        let active = voicemail::ActiveModel {
            id: Set(0), // auto-increment
            mailbox_id: Set(mailbox_id.to_string()),
            caller_id: Set(caller_id.to_string()),
            caller_name: Set(caller_name),
            call_id: Set(call_id.to_string()),
            recording_path: Set(recording_path.to_string()),
            duration_secs: Set(duration_secs),
            is_read: Set(false),
            is_urgent: Set(false),
            transcript_text: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };

        active.insert(db).await.map_err(|e| {
            anyhow!("Failed to save voicemail record: {}", e)
        })?;

        info!(
            mailbox_id,
            call_id,
            recording_path,
            duration_secs,
            "Voicemail record saved to database"
        );

        Ok(())
    }
}
