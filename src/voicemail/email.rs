use crate::config::VoicemailEmailConfig;
use anyhow::Result;
use tracing::warn;

/// Information about a voicemail message, used to build email notifications.
#[derive(Debug, Clone)]
pub struct VoicemailNotification {
    pub recipient_email: String,
    pub caller_id: String,
    pub caller_name: Option<String>,
    pub mailbox_id: String,
    pub duration_secs: i32,
    pub recording_path: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Sends voicemail email notifications via SMTP.
///
/// When the `email` feature is enabled, this uses the `lettre` crate to
/// build and deliver MIME emails. Without the feature, all methods are
/// no-ops that log a warning.
pub struct VoicemailEmailNotifier {
    #[allow(dead_code)]
    config: VoicemailEmailConfig,
}

impl VoicemailEmailNotifier {
    pub fn new(config: VoicemailEmailConfig) -> Self {
        Self { config }
    }

    /// Format the email subject by expanding `{caller}` placeholders in
    /// the configured subject template.
    #[allow(dead_code)]
    fn format_subject(&self, notification: &VoicemailNotification) -> String {
        let template = self
            .config
            .subject_template
            .as_deref()
            .unwrap_or("New voicemail from {caller}");

        let caller_display = notification
            .caller_name
            .as_deref()
            .unwrap_or(&notification.caller_id);

        template.replace("{caller}", caller_display)
    }

    /// Build the plain-text body of the notification email.
    #[allow(dead_code)]
    fn build_body(&self, notification: &VoicemailNotification) -> String {
        let caller_display = notification
            .caller_name
            .as_deref()
            .unwrap_or(&notification.caller_id);

        let minutes = notification.duration_secs / 60;
        let seconds = notification.duration_secs % 60;

        format!(
            "You have a new voicemail message.\n\
             \n\
             From: {} ({})\n\
             Mailbox: {}\n\
             Date: {}\n\
             Duration: {}:{:02}\n\
             \n\
             {}",
            caller_display,
            notification.caller_id,
            notification.mailbox_id,
            notification.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            minutes,
            seconds,
            if self.config.attach_audio {
                "The voicemail audio file is attached to this message."
            } else {
                "Log in to your voicemail system to listen to this message."
            }
        )
    }

    /// Send a voicemail notification email.
    ///
    /// When the `email` feature is enabled, this connects to the configured
    /// SMTP server and delivers the message. If `attach_audio` is true and
    /// the recording file exists, the WAV file is included as an attachment.
    ///
    /// Without the `email` feature this is a no-op that logs a warning.
    pub async fn send_notification(&self, notification: &VoicemailNotification) -> Result<()> {
        #[cfg(feature = "email")]
        {
            self.send_notification_impl(notification).await
        }

        #[cfg(not(feature = "email"))]
        {
            warn!(
                recipient = %notification.recipient_email,
                mailbox = %notification.mailbox_id,
                "Voicemail email notification skipped: 'email' feature not enabled. \
                 Rebuild with --features email to enable SMTP delivery."
            );
            Ok(())
        }
    }

    /// Real SMTP implementation, compiled only when the `email` feature is active.
    #[cfg(feature = "email")]
    async fn send_notification_impl(&self, notification: &VoicemailNotification) -> Result<()> {
        use lettre::{
            AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
            message::{Attachment, MultiPart, SinglePart, header::ContentType},
            transport::smtp::authentication::Credentials,
        };
        use tracing::info;

        let subject = self.format_subject(notification);
        let body_text = self.build_body(notification);

        // Start building the message
        let mut builder = lettre::Message::builder()
            .from(
                self.config
                    .from_address
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid from_address '{}': {}", self.config.from_address, e))?,
            )
            .to(notification
                .recipient_email
                .parse()
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Invalid recipient email '{}': {}",
                        notification.recipient_email,
                        e
                    )
                })?)
            .subject(subject);

        // Set the Date header
        builder = builder.date_now();

        let email = if self.config.attach_audio {
            // Try to read the recording file for attachment
            let recording_path = std::path::Path::new(&notification.recording_path);
            if recording_path.exists() {
                let file_bytes = tokio::fs::read(recording_path).await.map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to read voicemail recording '{}': {}",
                        notification.recording_path,
                        e
                    )
                })?;

                let filename = recording_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("voicemail.wav")
                    .to_string();

                let attachment = Attachment::new(filename)
                    .body(file_bytes, ContentType::parse("audio/wav").unwrap());

                builder.multipart(
                    MultiPart::mixed()
                        .singlepart(SinglePart::plain(body_text))
                        .singlepart(attachment),
                )?
            } else {
                warn!(
                    recording_path = %notification.recording_path,
                    "Voicemail recording file not found, sending notification without attachment"
                );
                builder.body(body_text)?
            }
        } else {
            builder.body(body_text)?
        };

        // Build the SMTP transport
        let mut transport_builder =
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.config.smtp_host)?
                .port(self.config.smtp_port);

        if let (Some(username), Some(password)) =
            (&self.config.smtp_username, &self.config.smtp_password)
        {
            transport_builder =
                transport_builder.credentials(Credentials::new(username.clone(), password.clone()));
        }

        let transport = transport_builder.build();

        transport.send(email).await.map_err(|e| {
            anyhow::anyhow!(
                "Failed to send voicemail notification email to '{}': {}",
                notification.recipient_email,
                e
            )
        })?;

        info!(
            recipient = %notification.recipient_email,
            mailbox = %notification.mailbox_id,
            caller = %notification.caller_id,
            "Voicemail email notification sent successfully"
        );

        Ok(())
    }
}
