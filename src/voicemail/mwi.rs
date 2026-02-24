//! MWI (Message Waiting Indicator) via SIP NOTIFY (RFC 3842).
//!
//! Sends unsolicited SIP NOTIFY messages to a user's registered phone(s)
//! when a new voicemail arrives, so the phone can show a message-waiting
//! light or icon.

use crate::models::voicemail;
use crate::proxy::locator::Locator;
use anyhow::Result;
use rsip::prelude::UntypedHeader;
use rsipstack::transaction::{
    key::{TransactionKey, TransactionRole},
    transaction::Transaction,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Counts of voicemail messages for a given mailbox.
#[derive(Debug, Clone, Default)]
pub struct VoicemailCounts {
    pub new_messages: i64,
    pub old_messages: i64,
}

/// Service responsible for sending MWI (Message Waiting Indicator) SIP NOTIFY
/// messages to registered user agents.
pub struct MwiService {
    endpoint_inner: rsipstack::transaction::endpoint::EndpointInnerRef,
    locator: Arc<Box<dyn Locator>>,
    db: Option<DatabaseConnection>,
}

impl MwiService {
    /// Create a new MwiService.
    ///
    /// - `endpoint_inner`: the SIP endpoint used to construct and send requests
    /// - `locator`: the registration locator to find user contacts
    /// - `db`: optional database connection for querying voicemail counts
    pub fn new(
        endpoint_inner: rsipstack::transaction::endpoint::EndpointInnerRef,
        locator: Arc<Box<dyn Locator>>,
        db: Option<DatabaseConnection>,
    ) -> Self {
        Self {
            endpoint_inner,
            locator,
            db,
        }
    }

    /// Query the database to get voicemail counts (new/unread and old/read)
    /// for a given mailbox. Returns default (0/0) if no database is configured.
    pub async fn get_voicemail_counts(&self, mailbox_id: &str) -> Result<VoicemailCounts> {
        let db = match &self.db {
            Some(db) => db,
            None => return Ok(VoicemailCounts::default()),
        };

        // Count unread (new) messages: is_read = false, deleted_at IS NULL
        let new_messages = voicemail::Entity::find()
            .filter(voicemail::Column::MailboxId.eq(mailbox_id))
            .filter(voicemail::Column::IsRead.eq(false))
            .filter(voicemail::Column::DeletedAt.is_null())
            .count(db)
            .await
            .unwrap_or(0) as i64;

        // Count read (old) messages: is_read = true, deleted_at IS NULL
        let old_messages = voicemail::Entity::find()
            .filter(voicemail::Column::MailboxId.eq(mailbox_id))
            .filter(voicemail::Column::IsRead.eq(true))
            .filter(voicemail::Column::DeletedAt.is_null())
            .count(db)
            .await
            .unwrap_or(0) as i64;

        Ok(VoicemailCounts {
            new_messages,
            old_messages,
        })
    }

    /// Send MWI NOTIFY to all registered contacts for a given mailbox.
    ///
    /// This sends an unsolicited NOTIFY (no SUBSCRIBE dialog needed) per
    /// RFC 3842. If the user is not registered, nothing happens.
    /// This is best-effort; errors are logged but not propagated.
    pub async fn send_mwi_notify(&self, mailbox_id: &str, realm: &str) {
        let counts = match self.get_voicemail_counts(mailbox_id).await {
            Ok(c) => c,
            Err(e) => {
                warn!(
                    mailbox_id,
                    error = %e,
                    "MWI: failed to query voicemail counts"
                );
                return;
            }
        };

        self.send_mwi_notify_with_counts(mailbox_id, realm, &counts)
            .await;
    }

    /// Send MWI NOTIFY with explicitly provided voicemail counts.
    pub async fn send_mwi_notify_with_counts(
        &self,
        mailbox_id: &str,
        realm: &str,
        counts: &VoicemailCounts,
    ) {
        // Look up registered contacts for this mailbox user
        let lookup_uri = match rsip::Uri::try_from(format!("sip:{}@{}", mailbox_id, realm).as_str())
        {
            Ok(uri) => uri,
            Err(e) => {
                warn!(
                    mailbox_id,
                    realm,
                    error = %e,
                    "MWI: failed to parse lookup URI"
                );
                return;
            }
        };

        let locations = match self.locator.lookup(&lookup_uri).await {
            Ok(locs) => locs,
            Err(e) => {
                debug!(
                    mailbox_id,
                    error = %e,
                    "MWI: user not registered, skipping notification"
                );
                return;
            }
        };

        if locations.is_empty() {
            debug!(
                mailbox_id,
                "MWI: no registered contacts found, skipping notification"
            );
            return;
        }

        // Build the message-summary body (RFC 3842)
        let messages_waiting = if counts.new_messages > 0 {
            "yes"
        } else {
            "no"
        };
        let body = format!(
            "Messages-Waiting: {}\r\n\
             Message-Account: sip:{}@{}\r\n\
             Voice-Message: {}/{} (0/0)\r\n",
            messages_waiting, mailbox_id, realm, counts.new_messages, counts.old_messages,
        );
        let body_bytes = body.into_bytes();

        for location in &locations {
            let contact_uri = &location.aor;
            let destination = location.destination.clone();

            if let Err(e) = self
                .send_single_mwi_notify(
                    mailbox_id,
                    realm,
                    contact_uri,
                    destination,
                    &body_bytes,
                )
                .await
            {
                warn!(
                    mailbox_id,
                    contact = %contact_uri,
                    error = %e,
                    "MWI: failed to send NOTIFY"
                );
            }
        }

        info!(
            mailbox_id,
            new = counts.new_messages,
            old = counts.old_messages,
            contacts = locations.len(),
            "MWI: sent NOTIFY to registered contacts"
        );
    }

    /// Send a single MWI NOTIFY to one registered contact.
    async fn send_single_mwi_notify(
        &self,
        mailbox_id: &str,
        realm: &str,
        contact_uri: &rsip::Uri,
        destination: Option<rsipstack::transport::SipAddr>,
        body: &[u8],
    ) -> Result<()> {
        // Build the SIP NOTIFY request
        let via = self
            .endpoint_inner
            .get_via(None, None)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        let from_uri =
            rsip::Uri::try_from(format!("sip:{}@{}", mailbox_id, realm).as_str())?;
        let from = rsip::typed::From {
            display_name: Some("Voicemail".to_string()),
            uri: from_uri,
            params: vec![rsip::Param::Tag(
                rsipstack::transaction::make_tag(),
            )],
        };

        let to = rsip::typed::To {
            display_name: None,
            uri: contact_uri.clone(),
            params: vec![],
        };

        let mut request = self.endpoint_inner.make_request(
            rsip::Method::Notify,
            contact_uri.clone(),
            via,
            from,
            to,
            1,
            None,
        );

        // Add MWI-specific headers
        request
            .headers
            .push(rsip::Header::Event(rsip::headers::Event::new(
                "message-summary",
            )));
        request.headers.push(rsip::Header::SubscriptionState(
            rsip::headers::SubscriptionState::new("active"),
        ));
        request
            .headers
            .push(rsip::Header::ContentType(rsip::headers::ContentType::from(
                "application/simple-message-summary",
            )));

        // Set the body
        request.body = body.to_vec();

        // Create and send a client transaction
        let key = TransactionKey::from_request(&request, TransactionRole::Client)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        let mut tx =
            Transaction::new_client(key, request, self.endpoint_inner.clone(), None);

        // If we have a known destination for this contact, use it
        if let Some(dest) = destination {
            tx.destination = Some(dest);
        }

        tx.send().await.map_err(|e| anyhow::anyhow!("{:?}", e))?;

        // Wait briefly for the response (best-effort, don't block long)
        match tokio::time::timeout(std::time::Duration::from_secs(5), tx.receive()).await {
            Ok(Some(msg)) => {
                if let rsip::SipMessage::Response(resp) = msg {
                    debug!(
                        mailbox_id,
                        status = %resp.status_code,
                        "MWI: received response to NOTIFY"
                    );
                }
            }
            Ok(None) => {
                debug!(
                    mailbox_id,
                    "MWI: no response received for NOTIFY"
                );
            }
            Err(_) => {
                debug!(mailbox_id, "MWI: NOTIFY response timed out (best-effort)");
            }
        }

        Ok(())
    }
}
