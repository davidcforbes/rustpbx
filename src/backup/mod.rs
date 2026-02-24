use crate::config::BackupConfig;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::time;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

/// Status of the backup service, queryable via the AMI endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStatus {
    pub last_backup_time: Option<DateTime<Utc>>,
    pub last_backup_size_bytes: Option<u64>,
    pub last_backup_path: Option<String>,
    pub last_error: Option<String>,
    pub total_backups: u64,
    pub total_failures: u64,
    pub next_scheduled: Option<DateTime<Utc>>,
    pub healthy: bool,
}

impl Default for BackupStatus {
    fn default() -> Self {
        Self {
            last_backup_time: None,
            last_backup_size_bytes: None,
            last_backup_path: None,
            last_error: None,
            total_backups: 0,
            total_failures: 0,
            next_scheduled: None,
            healthy: true,
        }
    }
}

/// Shared state for the backup service, accessible from AMI handlers.
#[derive(Clone)]
pub struct BackupService {
    pub status: Arc<RwLock<BackupStatus>>,
    pub config: Arc<RwLock<BackupConfig>>,
    /// The database URL from the main config, used to locate the SQLite file.
    pub database_url: String,
    /// Path to recordings, used when include_recordings is enabled.
    pub recorder_path: String,
}

impl BackupService {
    pub fn new(config: &BackupConfig, database_url: &str, recorder_path: &str) -> Self {
        Self {
            status: Arc::new(RwLock::new(BackupStatus::default())),
            config: Arc::new(RwLock::new(config.clone())),
            database_url: database_url.to_string(),
            recorder_path: recorder_path.to_string(),
        }
    }

    /// Start the background scheduler loop. This should be spawned as a tokio task.
    pub async fn run_scheduler(&self, token: CancellationToken) {
        let interval_secs = self.parse_schedule_interval();
        info!(
            interval_secs,
            "Backup scheduler started (interval-based scheduling)"
        );

        // Compute and store the first scheduled time
        {
            let next = Utc::now() + chrono::Duration::seconds(interval_secs as i64);
            if let Ok(mut status) = self.status.write() {
                status.next_scheduled = Some(next);
            }
        }

        let mut interval = time::interval(time::Duration::from_secs(interval_secs));
        // The first tick completes immediately; skip it so we don't back up on startup.
        interval.tick().await;

        loop {
            tokio::select! {
                _ = interval.tick() => {}
                _ = token.cancelled() => {
                    info!("Backup scheduler shutting down");
                    return;
                }
            }

            let config = {
                let guard = self.config.read().unwrap();
                guard.clone()
            };

            if !config.enabled {
                continue;
            }

            info!("Starting scheduled backup");
            self.perform_backup_internal(&config).await;

            // Update next scheduled time
            if let Ok(mut status) = self.status.write() {
                status.next_scheduled =
                    Some(Utc::now() + chrono::Duration::seconds(interval_secs as i64));
            }
        }
    }

    /// Manually trigger a backup (called from the AMI endpoint).
    pub async fn trigger_backup(&self) -> Result<String, String> {
        let config = {
            let guard = self.config.read().unwrap();
            guard.clone()
        };

        info!("Manual backup triggered via API");
        let result = self.perform_backup(&config).await;
        match result {
            Ok(path) => Ok(path),
            Err(e) => Err(e),
        }
    }

    /// Get the current backup status.
    pub fn get_status(&self) -> BackupStatus {
        self.status.read().unwrap().clone()
    }

    /// Parse the schedule_cron field into an interval in seconds.
    /// Supports simple cron-like patterns or direct interval strings.
    /// For simplicity, we parse common patterns:
    ///   "0 * * * *"    -> every hour (3600s)
    ///   "*/5 * * * *"  -> every 5 minutes (300s)
    ///   "0 0 * * *"    -> every day (86400s)
    ///   "0 */2 * * *"  -> every 2 hours (7200s)
    /// Falls back to hourly if not recognized.
    fn parse_schedule_interval(&self) -> u64 {
        let cron = self.config.read().unwrap().schedule_cron.clone();
        parse_cron_to_interval(&cron)
    }

    async fn perform_backup_internal(&self, config: &BackupConfig) {
        match self.perform_backup(config).await {
            Ok(path) => {
                info!(path, "Scheduled backup completed successfully");
            }
            Err(e) => {
                error!(error = %e, "Scheduled backup failed");
                if config.notify_on_failure {
                    warn!("Backup failure notification: {}", e);
                }
            }
        }
    }

    async fn perform_backup(&self, config: &BackupConfig) -> Result<String, String> {
        let backup_dir = PathBuf::from(&config.backup_dir);

        // Ensure backup directory exists
        if let Err(e) = tokio::fs::create_dir_all(&backup_dir).await {
            let msg = format!("Failed to create backup directory {:?}: {}", backup_dir, e);
            self.record_failure(&msg);
            return Err(msg);
        }

        // Perform database backup
        let db_backup_path = match self.backup_database(&backup_dir).await {
            Ok(path) => path,
            Err(e) => {
                let msg = format!("Database backup failed: {}", e);
                self.record_failure(&msg);
                return Err(msg);
            }
        };

        // Optionally copy recordings
        if config.include_recordings {
            if let Err(e) = self.backup_recordings(&backup_dir).await {
                warn!(error = %e, "Recording backup failed (db backup succeeded)");
            }
        }

        // Rotate old backups
        if let Err(e) = self.rotate_backups(&backup_dir, config.retention_days).await {
            warn!(error = %e, "Backup rotation failed");
        }

        // Record success
        let file_size = tokio::fs::metadata(&db_backup_path)
            .await
            .map(|m| m.len())
            .unwrap_or(0);

        if let Ok(mut status) = self.status.write() {
            status.last_backup_time = Some(Utc::now());
            status.last_backup_size_bytes = Some(file_size);
            status.last_backup_path = Some(db_backup_path.to_string_lossy().to_string());
            status.last_error = None;
            status.total_backups += 1;
            status.healthy = true;
        }

        Ok(db_backup_path.to_string_lossy().to_string())
    }

    /// Back up the SQLite database using a file copy.
    /// SQLite supports safe file-copy backup when using WAL mode or when the
    /// database is idle. We use tokio::fs::copy for async I/O.
    async fn backup_database(&self, backup_dir: &Path) -> Result<PathBuf, String> {
        let db_path = sqlite_path_from_url(&self.database_url)?;

        if !Path::new(&db_path).exists() {
            return Err(format!("Database file not found: {}", db_path));
        }

        let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let backup_filename = format!("rustpbx-{}.sqlite3", timestamp);
        let dest = backup_dir.join(&backup_filename);

        // First try using sqlite3 CLI for a safe online backup
        match try_sqlite3_backup(&db_path, &dest).await {
            Ok(()) => {
                info!(
                    src = %db_path,
                    dest = %dest.display(),
                    "Database backed up via sqlite3 .backup"
                );
                return Ok(dest);
            }
            Err(e) => {
                warn!(
                    error = %e,
                    "sqlite3 CLI backup failed, falling back to file copy"
                );
            }
        }

        // Fallback: simple file copy
        match tokio::fs::copy(&db_path, &dest).await {
            Ok(bytes) => {
                info!(
                    src = %db_path,
                    dest = %dest.display(),
                    bytes,
                    "Database backed up via file copy"
                );
                Ok(dest)
            }
            Err(e) => Err(format!(
                "Failed to copy database {} -> {}: {}",
                db_path,
                dest.display(),
                e
            )),
        }
    }

    /// Copy recordings to the backup directory.
    async fn backup_recordings(&self, backup_dir: &Path) -> Result<(), String> {
        let recordings_src = Path::new(&self.recorder_path);
        if !recordings_src.exists() {
            info!(
                path = %self.recorder_path,
                "Recordings directory does not exist, skipping"
            );
            return Ok(());
        }

        let recordings_dest = backup_dir.join("recordings");
        if let Err(e) = tokio::fs::create_dir_all(&recordings_dest).await {
            return Err(format!(
                "Failed to create recordings backup dir: {}",
                e
            ));
        }

        // Walk the recordings directory and copy .wav files
        let mut entries = match tokio::fs::read_dir(recordings_src).await {
            Ok(e) => e,
            Err(e) => {
                return Err(format!(
                    "Failed to read recordings directory {}: {}",
                    self.recorder_path, e
                ));
            }
        };

        let mut copied = 0u64;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name() {
                    let dest_file = recordings_dest.join(name);
                    // Only copy if not already present in backup
                    if !dest_file.exists() {
                        if let Err(e) = tokio::fs::copy(&path, &dest_file).await {
                            warn!(
                                src = %path.display(),
                                error = %e,
                                "Failed to copy recording file"
                            );
                        } else {
                            copied += 1;
                        }
                    }
                }
            }
        }

        info!(copied, "Recordings backup completed");
        Ok(())
    }

    /// Remove backup files older than retention_days.
    async fn rotate_backups(&self, backup_dir: &Path, retention_days: u32) -> Result<(), String> {
        let mut entries = match tokio::fs::read_dir(backup_dir).await {
            Ok(e) => e,
            Err(e) => return Err(format!("Failed to read backup directory: {}", e)),
        };

        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        let mut removed = 0u64;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            // Only rotate .sqlite3 backup files, not the recordings subdirectory
            if path.is_file()
                && path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == "sqlite3")
                    .unwrap_or(false)
            {
                if let Ok(metadata) = tokio::fs::metadata(&path).await {
                    if let Ok(modified) = metadata.modified() {
                        let modified_dt: DateTime<Utc> = modified.into();
                        if modified_dt < cutoff {
                            if let Err(e) = tokio::fs::remove_file(&path).await {
                                warn!(
                                    path = %path.display(),
                                    error = %e,
                                    "Failed to remove old backup"
                                );
                            } else {
                                info!(path = %path.display(), "Removed old backup");
                                removed += 1;
                            }
                        }
                    }
                }
            }
        }

        if removed > 0 {
            info!(removed, "Backup rotation completed");
        }
        Ok(())
    }

    fn record_failure(&self, msg: &str) {
        if let Ok(mut status) = self.status.write() {
            status.last_error = Some(msg.to_string());
            status.total_failures += 1;
            status.healthy = false;
        }
    }
}

/// Extract the SQLite file path from a database URL like "sqlite://rustpbx.sqlite3".
fn sqlite_path_from_url(url: &str) -> Result<String, String> {
    if let Some(path) = url.strip_prefix("sqlite://") {
        if path.is_empty() {
            return Err("Empty SQLite path in database_url".to_string());
        }
        Ok(path.to_string())
    } else {
        Err(format!(
            "Unsupported database URL for backup (only SQLite is supported): {}",
            url
        ))
    }
}

/// Try to perform a backup using the sqlite3 CLI tool's .backup command.
/// This is the safest method as it uses SQLite's online backup API.
async fn try_sqlite3_backup(db_path: &str, dest: &Path) -> Result<(), String> {
    let dest_str = dest.to_string_lossy().to_string();

    let output = tokio::process::Command::new("sqlite3")
        .arg(db_path)
        .arg(format!(".backup '{}'", dest_str))
        .output()
        .await
        .map_err(|e| format!("Failed to run sqlite3: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("sqlite3 backup failed: {}", stderr.trim()))
    }
}

/// Parse a simple cron expression into an interval in seconds.
/// Supports common patterns; falls back to 3600 (hourly) for unrecognized expressions.
fn parse_cron_to_interval(cron: &str) -> u64 {
    let parts: Vec<&str> = cron.trim().split_whitespace().collect();
    if parts.len() != 5 {
        warn!(
            cron,
            "Unrecognized cron expression, defaulting to hourly (3600s)"
        );
        return 3600;
    }

    let (minute, hour, _day, _month, _dow) = (parts[0], parts[1], parts[2], parts[3], parts[4]);

    // "*/N * * * *" -> every N minutes
    if let Some(n_str) = minute.strip_prefix("*/") {
        if let Ok(n) = n_str.parse::<u64>() {
            if n > 0 && n <= 60 {
                return n * 60;
            }
        }
    }

    // "0 */N * * *" -> every N hours
    if minute == "0" {
        if let Some(n_str) = hour.strip_prefix("*/") {
            if let Ok(n) = n_str.parse::<u64>() {
                if n > 0 && n <= 24 {
                    return n * 3600;
                }
            }
        }
    }

    // "0 * * * *" -> every hour
    if minute == "0" && hour == "*" {
        return 3600;
    }

    // "0 0 * * *" -> every day
    if minute == "0" && hour == "0" {
        return 86400;
    }

    // "N * * * *" where N is a fixed minute -> every hour
    if minute.parse::<u64>().is_ok() && hour == "*" {
        return 3600;
    }

    // "0 N * * *" where N is a fixed hour -> every day
    if minute == "0" && hour.parse::<u64>().is_ok() {
        return 86400;
    }

    warn!(
        cron,
        "Could not parse cron expression into interval, defaulting to hourly (3600s)"
    );
    3600
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cron_to_interval() {
        assert_eq!(parse_cron_to_interval("0 * * * *"), 3600);
        assert_eq!(parse_cron_to_interval("*/5 * * * *"), 300);
        assert_eq!(parse_cron_to_interval("*/15 * * * *"), 900);
        assert_eq!(parse_cron_to_interval("0 */2 * * *"), 7200);
        assert_eq!(parse_cron_to_interval("0 0 * * *"), 86400);
        assert_eq!(parse_cron_to_interval("30 * * * *"), 3600);
        assert_eq!(parse_cron_to_interval("0 3 * * *"), 86400);
        // Fallback for unrecognized
        assert_eq!(parse_cron_to_interval("garbage"), 3600);
    }

    #[test]
    fn test_sqlite_path_from_url() {
        assert_eq!(
            sqlite_path_from_url("sqlite://rustpbx.sqlite3").unwrap(),
            "rustpbx.sqlite3"
        );
        assert_eq!(
            sqlite_path_from_url("sqlite:///var/db/test.db").unwrap(),
            "/var/db/test.db"
        );
        assert!(sqlite_path_from_url("postgres://localhost/db").is_err());
        assert!(sqlite_path_from_url("sqlite://").is_err());
    }

    #[test]
    fn test_backup_config_defaults() {
        let config = crate::config::BackupConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.schedule_cron, "0 * * * *");
        assert_eq!(config.backup_dir, "./backups");
        assert_eq!(config.retention_days, 30);
        assert!(!config.include_recordings);
        assert!(config.notify_on_failure);
    }
}
