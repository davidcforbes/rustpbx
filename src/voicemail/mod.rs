pub mod email;
pub mod menu;
pub mod mwi;

use crate::config::VoicemailConfig;
use crate::media::recorder::RecorderOption;
use crate::models::voicemail;
use crate::models::voicemail_greeting;
use anyhow::{Result, anyhow};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use sea_orm::sea_query::Order;
use std::path::PathBuf;
use tracing::{info, warn};

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
    /// First checks the database for an active custom greeting uploaded by the
    /// user. If none exists (or there is no database), falls back to a
    /// per-mailbox WAV file on disk, and finally to the system default greeting.
    /// Returns None if no greeting file exists anywhere.
    pub async fn resolve_greeting_path(&self, mailbox_id: &str) -> Option<String> {
        // 1. Check database for an active custom greeting
        if let Some(db) = &self.db {
            match voicemail_greeting::Entity::find()
                .filter(voicemail_greeting::Column::MailboxId.eq(mailbox_id))
                .filter(voicemail_greeting::Column::IsActive.eq(true))
                .order_by(voicemail_greeting::Column::CreatedAt, Order::Desc)
                .one(db)
                .await
            {
                Ok(Some(greeting)) => {
                    let path = PathBuf::from(&greeting.recording_path);
                    if path.exists() {
                        info!(
                            mailbox_id,
                            path = %greeting.recording_path,
                            "Using custom greeting from database"
                        );
                        return Some(greeting.recording_path);
                    }
                    warn!(
                        mailbox_id,
                        path = %greeting.recording_path,
                        "Active greeting in database but file not found on disk"
                    );
                }
                Ok(None) => {
                    // No active greeting in DB, fall through to filesystem
                }
                Err(e) => {
                    warn!(
                        mailbox_id,
                        error = %e,
                        "Failed to query greeting from database, falling back to filesystem"
                    );
                }
            }
        }

        // 2. Try per-mailbox greeting file on disk
        let custom = PathBuf::from(&self.config.greeting_path)
            .join(format!("{}.wav", mailbox_id));
        if custom.exists() {
            return Some(custom.to_string_lossy().to_string());
        }

        // 3. Try system default greeting
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

    /// List voicemail messages for a mailbox, ordered by newest first.
    ///
    /// Only returns non-deleted messages (where deleted_at is NULL).
    pub async fn get_messages(&self, mailbox_id: &str) -> Result<Vec<voicemail::Model>> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| anyhow!("No database configured"))?;

        let messages = voicemail::Entity::find()
            .filter(voicemail::Column::MailboxId.eq(mailbox_id))
            .filter(voicemail::Column::DeletedAt.is_null())
            .order_by(voicemail::Column::CreatedAt, Order::Desc)
            .all(db)
            .await
            .map_err(|e| anyhow!("Failed to query voicemails: {}", e))?;

        Ok(messages)
    }

    /// Mark a voicemail message as read.
    pub async fn mark_read(&self, message_id: i64) -> Result<()> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| anyhow!("No database configured"))?;

        let msg = voicemail::Entity::find_by_id(message_id)
            .one(db)
            .await
            .map_err(|e| anyhow!("Failed to find voicemail {}: {}", message_id, e))?
            .ok_or_else(|| anyhow!("Voicemail message {} not found", message_id))?;

        let mut active: voicemail::ActiveModel = msg.into();
        active.is_read = Set(true);
        active.updated_at = Set(Utc::now());

        active
            .update(db)
            .await
            .map_err(|e| anyhow!("Failed to mark voicemail {} as read: {}", message_id, e))?;

        info!(message_id, "Voicemail message marked as read");
        Ok(())
    }

    /// Soft-delete a voicemail message by setting its deleted_at timestamp.
    pub async fn delete_message(&self, message_id: i64) -> Result<()> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| anyhow!("No database configured"))?;

        let msg = voicemail::Entity::find_by_id(message_id)
            .one(db)
            .await
            .map_err(|e| anyhow!("Failed to find voicemail {}: {}", message_id, e))?
            .ok_or_else(|| anyhow!("Voicemail message {} not found", message_id))?;

        let mut active: voicemail::ActiveModel = msg.into();
        active.deleted_at = Set(Some(Utc::now()));
        active.updated_at = Set(Utc::now());

        active
            .update(db)
            .await
            .map_err(|e| anyhow!("Failed to delete voicemail {}: {}", message_id, e))?;

        info!(message_id, "Voicemail message soft-deleted");
        Ok(())
    }

    /// Send an email notification for a new voicemail.
    ///
    /// This should be called after `save_voicemail_record()`. The
    /// `recipient_email` is typically obtained from the callee's `SipUser`
    /// record. If no email config is present or no recipient email is
    /// provided, this is a silent no-op.
    pub async fn send_email_notification(
        &self,
        recipient_email: Option<&str>,
        mailbox_id: &str,
        caller_id: &str,
        caller_name: Option<String>,
        recording_path: &str,
        duration_secs: i32,
    ) {
        let email_config = match &self.config.email_notifications {
            Some(cfg) => cfg.clone(),
            None => return,
        };

        let recipient = match recipient_email {
            Some(em) if !em.is_empty() => em.to_string(),
            _ => {
                info!(
                    mailbox_id,
                    "No email address for mailbox owner, skipping voicemail email notification"
                );
                return;
            }
        };

        let notification = email::VoicemailNotification {
            recipient_email: recipient,
            caller_id: caller_id.to_string(),
            caller_name,
            mailbox_id: mailbox_id.to_string(),
            duration_secs,
            recording_path: recording_path.to_string(),
            timestamp: Utc::now(),
        };

        let notifier = email::VoicemailEmailNotifier::new(email_config);
        if let Err(e) = notifier.send_notification(&notification).await {
            warn!(
                mailbox_id,
                error = %e,
                "Failed to send voicemail email notification"
            );
        }
    }
}
