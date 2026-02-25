use anyhow::Result;
use rsip::{
    Header, Transport,
    headers::auth::Algorithm,
    prelude::{HeadersExt, ToTypedHeader},
    typed::Authorization,
};
use rsipstack::{
    transaction::transaction::Transaction,
    transport::{SipAddr, SipConnection},
};
use serde::{Deserialize, Serialize};

use super::{CallForwardingConfig, CallForwardingMode, TransferEndpoint};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SipUser {
    #[serde(default)]
    pub id: u64,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub username: String,
    pub password: Option<String>,
    pub realm: Option<String>,
    pub departments: Option<Vec<String>>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub note: Option<String>,
    #[serde(default)]
    pub allow_guest_calls: bool,
    #[serde(default)]
    pub call_forwarding_mode: Option<String>,
    #[serde(default)]
    pub call_forwarding_destination: Option<String>,
    #[serde(default)]
    pub call_forwarding_timeout: Option<i32>,
    /// Whether voicemail is enabled for this user (default true)
    #[serde(default = "default_voicemail_enabled")]
    pub voicemail_enabled: bool,
    /// From the original INVITE
    #[serde(skip)]
    pub origin_contact: Option<rsip::typed::Contact>,
    /// Current contact (may be updated by REGISTER)
    #[serde(skip)]
    pub contact: Option<rsip::typed::Contact>,
    #[serde(skip)]
    pub from: Option<rsip::Uri>,
    #[serde(skip)]
    pub destination: Option<SipAddr>,
    #[serde(default = "default_is_support_webrtc")]
    pub is_support_webrtc: bool,
}

impl ToString for SipUser {
    fn to_string(&self) -> String {
        if let Some(realm) = &self.realm {
            format!("{}@{}", self.username, realm)
        } else {
            self.username.clone()
        }
    }
}

fn default_enabled() -> bool {
    true
}

fn default_is_support_webrtc() -> bool {
    false
}

fn default_voicemail_enabled() -> bool {
    true
}

impl Default for SipUser {
    fn default() -> Self {
        Self {
            id: 0,
            enabled: true,
            username: "".to_string(),
            password: None,
            realm: None,
            origin_contact: None,
            contact: None,
            from: None,
            destination: None,
            is_support_webrtc: false,
            departments: None,
            display_name: None,
            email: None,
            phone: None,
            note: None,
            allow_guest_calls: false,
            call_forwarding_mode: None,
            call_forwarding_destination: None,
            call_forwarding_timeout: None,
            voicemail_enabled: true,
        }
    }
}

impl SipUser {
    pub fn get_contact_username(&self) -> String {
        match self.origin_contact {
            Some(ref contact) => contact.uri.user().unwrap_or_default().to_string(),
            None => self.username.clone(),
        }
    }
    pub fn merge_with(&mut self, other: &SipUser) {
        if self.id == 0 {
            self.id = other.id;
        }
        if self.password.is_none() {
            self.password = other.password.clone();
        }
        if self.realm.is_none() {
            self.realm = other.realm.clone();
        }
        if self.departments.is_none() {
            self.departments = other.departments.clone();
        }
        if self.display_name.is_none() {
            self.display_name = other.display_name.clone();
        }
        if self.email.is_none() {
            self.email = other.email.clone();
        }
        if self.phone.is_none() {
            self.phone = other.phone.clone();
        }
        if self.note.is_none() {
            self.note = other.note.clone();
        }
        if !self.allow_guest_calls {
            self.allow_guest_calls = other.allow_guest_calls;
        }
        if self.origin_contact.is_none() {
            self.origin_contact = other.origin_contact.clone();
        }
        if self.contact.is_none() {
            self.contact = other.contact.clone();
        }
        if self.from.is_none() {
            self.from = other.from.clone();
        }
        if self.destination.is_none() {
            self.destination = other.destination.clone();
        }
        if !self.is_support_webrtc {
            self.is_support_webrtc = other.is_support_webrtc;
        }
    }

    pub fn forwarding_config(&self) -> Option<CallForwardingConfig> {
        let mode_text = self
            .call_forwarding_mode
            .as_deref()
            .map(|value| value.trim().to_lowercase())?;
        if mode_text.is_empty() || mode_text == "none" {
            return None;
        }

        let destination = self
            .call_forwarding_destination
            .as_deref()
            .map(|value| value.trim())?;
        if destination.is_empty() {
            return None;
        }

        let endpoint = TransferEndpoint::parse(destination)?;

        let mode = match mode_text.as_str() {
            "always" => CallForwardingMode::Always,
            "when_busy" | "busy" => CallForwardingMode::WhenBusy,
            "when_not_answered" | "no_answer" => CallForwardingMode::WhenNoAnswer,
            _ => return None,
        };

        let timeout_secs = self
            .call_forwarding_timeout
            .map(|value| CallForwardingConfig::clamp_timeout(value as i64))
            .unwrap_or(super::CALL_FORWARDING_TIMEOUT_DEFAULT_SECS);

        Some(CallForwardingConfig::new(mode, endpoint, timeout_secs))
    }

    fn build_contact(&mut self, tx: &Transaction) {
        let addr = match tx.endpoint_inner.get_addrs().first() {
            Some(addr) => addr.clone(),
            None => return,
        };

        let mut contact_params = vec![];
        match addr.r#type {
            Some(rsip::Transport::Udp) | None => {}
            Some(t) => {
                contact_params.push(rsip::Param::Transport(t));
            }
        }
        let contact = rsip::typed::Contact {
            display_name: None,
            uri: rsip::Uri {
                scheme: addr.r#type.map(|t| t.sip_scheme()),
                auth: Some(rsip::Auth {
                    user: self.get_contact_username(),
                    password: None,
                }),
                host_with_port: addr.addr.clone(),
                ..Default::default()
            },
            params: contact_params,
        };
        self.contact = Some(contact);
    }

    pub fn auth_digest(&self, algorithm: Algorithm) -> String {
        use md5::{Digest, Md5};
        use sha2::{Sha256, Sha512};
        let value = format!(
            "{}:{}:{}",
            self.username,
            self.realm.as_ref().unwrap_or(&"".to_string()),
            self.password.as_ref().unwrap_or(&"".to_string()),
        );
        match algorithm {
            Algorithm::Md5 | Algorithm::Md5Sess => {
                let mut hasher = Md5::new();
                hasher.update(value);
                format!("{:x}", hasher.finalize())
            }
            Algorithm::Sha256 | Algorithm::Sha256Sess => {
                let mut hasher = Sha256::new();
                hasher.update(value);
                format!("{:x}", hasher.finalize())
            }
            Algorithm::Sha512 | Algorithm::Sha512Sess => {
                let mut hasher = Sha512::new();
                hasher.update(value);
                format!("{:x}", hasher.finalize())
            }
        }
    }
}

impl TryFrom<&Transaction> for SipUser {
    type Error = anyhow::Error;

    fn try_from(tx: &Transaction) -> Result<Self, Self::Error> {
        let from_header = tx.original.from_header()?;
        let from_uri = from_header.uri()?;
        let from_display_name = from_header
            .typed()
            .ok()
            .and_then(|h| h.display_name)
            .map(|s| s.to_string());

        let (username, realm) = match check_authorization_headers(&tx.original) {
            Ok(Some((user, _))) => (user.username, user.realm),
            _ => {
                let username = from_uri.user().unwrap_or_default().to_string();
                let realm = from_uri.host().to_string();
                let realm = if let Some(port) = from_uri.port() {
                    Some(format!("{}:{}", realm, port))
                } else {
                    Some(realm)
                };
                (username, realm)
            }
        };

        let origin_contact = match tx.original.contact_header() {
            Ok(contact) => contact.typed().ok(),
            Err(_) => None,
        };
        // Use rsipstack's via_received functionality to get destination
        let via_header = tx.original.via_header()?;
        let (via_transport, destination_addr) = SipConnection::parse_target_from_via(via_header)
            .map_err(|e| anyhow::anyhow!("failed to parse via header: {:?}", e))?;

        let destination = SipAddr {
            r#type: Some(via_transport),
            addr: destination_addr,
        };

        let is_support_webrtc = matches!(via_transport, Transport::Wss | Transport::Ws);

        let mut u = SipUser {
            id: 0,
            username,
            password: None,
            enabled: true,
            realm,
            origin_contact,
            contact: None,
            from: Some(from_uri),
            destination: Some(destination),
            is_support_webrtc,
            call_forwarding_mode: None,
            call_forwarding_destination: None,
            call_forwarding_timeout: None,
            departments: None,
            display_name: from_display_name,
            email: None,
            phone: None,
            note: None,
            allow_guest_calls: false,
            voicemail_enabled: true,
        };
        u.build_contact(tx);
        Ok(u)
    }
}

pub fn check_authorization_headers(
    req: &rsip::Request,
) -> Result<Option<(SipUser, Authorization)>> {
    // First try Authorization header (for backward compatibility with existing tests)
    if let Some(auth_header) = rsip::header_opt!(req.headers.iter(), Header::Authorization) {
        let challenge = auth_header.typed()?;
        let user = SipUser {
            username: challenge.username.to_string(),
            realm: Some(challenge.realm.to_string()),
            ..Default::default()
        };
        return Ok(Some((user, challenge)));
    }
    // Then try Proxy-Authorization header
    if let Some(proxy_auth_header) =
        rsip::header_opt!(req.headers.iter(), Header::ProxyAuthorization)
    {
        let challenge = proxy_auth_header.typed()?.0;
        let user = SipUser {
            username: challenge.username.to_string(),
            realm: Some(challenge.realm.to_string()),
            ..Default::default()
        };
        return Ok(Some((user, challenge)));
    }

    Ok(None)
}

/// Lightweight credential record used by `find_credentials_for_callee`.
#[derive(Clone, Debug)]
pub struct SipCredential {
    pub username: String,
    pub password: String,
    pub realm: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsip::headers::auth::Algorithm;
    use std::time::Duration;

    // ── SipUser defaults ─────────────────────────────────────────────────

    #[test]
    fn test_sip_user_default() {
        let user = SipUser::default();
        assert_eq!(user.id, 0);
        assert!(user.enabled);
        assert_eq!(user.username, "");
        assert!(user.password.is_none());
        assert!(user.realm.is_none());
        assert!(!user.allow_guest_calls);
        assert!(user.voicemail_enabled);
        assert!(!user.is_support_webrtc);
    }

    // ── Serialization / Deserialization ──────────────────────────────────

    #[test]
    fn test_sip_user_deserialize_minimal() {
        let json = r#"{"username": "1001"}"#;
        let user: SipUser = serde_json::from_str(json).unwrap();
        assert_eq!(user.username, "1001");
        assert!(user.enabled); // default_enabled
        assert!(user.voicemail_enabled); // default_voicemail_enabled
    }

    #[test]
    fn test_sip_user_deserialize_full() {
        let json = r#"{
            "username": "1001",
            "password": "secret",
            "realm": "example.com",
            "enabled": false,
            "display_name": "Alice",
            "email": "alice@example.com",
            "allow_guest_calls": true,
            "call_forwarding_mode": "always",
            "call_forwarding_destination": "1002",
            "call_forwarding_timeout": 45
        }"#;
        let user: SipUser = serde_json::from_str(json).unwrap();
        assert_eq!(user.username, "1001");
        assert_eq!(user.password.as_deref(), Some("secret"));
        assert_eq!(user.realm.as_deref(), Some("example.com"));
        assert!(!user.enabled);
        assert_eq!(user.display_name.as_deref(), Some("Alice"));
        assert!(user.allow_guest_calls);
        assert_eq!(user.call_forwarding_mode.as_deref(), Some("always"));
        assert_eq!(user.call_forwarding_destination.as_deref(), Some("1002"));
        assert_eq!(user.call_forwarding_timeout, Some(45));
    }

    #[test]
    fn test_sip_user_serialize_roundtrip() {
        let user = SipUser {
            username: "test".to_string(),
            password: Some("pass".to_string()),
            realm: Some("realm.io".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_string(&user).unwrap();
        let parsed: SipUser = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.username, user.username);
        assert_eq!(parsed.password, user.password);
        assert_eq!(parsed.realm, user.realm);
    }

    // ── to_string ────────────────────────────────────────────────────────

    #[test]
    fn test_sip_user_to_string_with_realm() {
        let user = SipUser {
            username: "1001".to_string(),
            realm: Some("example.com".to_string()),
            ..Default::default()
        };
        assert_eq!(user.to_string(), "1001@example.com");
    }

    #[test]
    fn test_sip_user_to_string_without_realm() {
        let user = SipUser {
            username: "1001".to_string(),
            ..Default::default()
        };
        assert_eq!(user.to_string(), "1001");
    }

    // ── get_contact_username ─────────────────────────────────────────────

    #[test]
    fn test_get_contact_username_no_contact_returns_username() {
        let user = SipUser {
            username: "1001".to_string(),
            ..Default::default()
        };
        assert_eq!(user.get_contact_username(), "1001");
    }

    // ── merge_with ───────────────────────────────────────────────────────

    #[test]
    fn test_merge_with_fills_missing_fields() {
        let mut user = SipUser {
            username: "1001".to_string(),
            ..Default::default()
        };
        let other = SipUser {
            id: 42,
            username: "other".to_string(),
            password: Some("secret".to_string()),
            realm: Some("example.com".to_string()),
            display_name: Some("Bob".to_string()),
            email: Some("bob@example.com".to_string()),
            phone: Some("+1234567890".to_string()),
            note: Some("VIP customer".to_string()),
            departments: Some(vec!["sales".to_string()]),
            ..Default::default()
        };
        user.merge_with(&other);
        assert_eq!(user.id, 42);
        assert_eq!(user.password.as_deref(), Some("secret"));
        assert_eq!(user.realm.as_deref(), Some("example.com"));
        assert_eq!(user.display_name.as_deref(), Some("Bob"));
        assert_eq!(user.email.as_deref(), Some("bob@example.com"));
        assert_eq!(user.phone.as_deref(), Some("+1234567890"));
        assert_eq!(user.note.as_deref(), Some("VIP customer"));
        assert_eq!(user.departments, Some(vec!["sales".to_string()]));
    }

    #[test]
    fn test_merge_with_does_not_overwrite_existing() {
        let mut user = SipUser {
            id: 10,
            username: "1001".to_string(),
            password: Some("existing".to_string()),
            realm: Some("mine.com".to_string()),
            ..Default::default()
        };
        let other = SipUser {
            id: 42,
            username: "other".to_string(),
            password: Some("other_secret".to_string()),
            realm: Some("other.com".to_string()),
            ..Default::default()
        };
        user.merge_with(&other);
        // Existing values should NOT be overwritten
        assert_eq!(user.id, 10);
        assert_eq!(user.password.as_deref(), Some("existing"));
        assert_eq!(user.realm.as_deref(), Some("mine.com"));
    }

    #[test]
    fn test_merge_with_allow_guest_calls_flag() {
        let mut user = SipUser {
            username: "1001".to_string(),
            allow_guest_calls: false,
            ..Default::default()
        };
        let other = SipUser {
            username: "other".to_string(),
            allow_guest_calls: true,
            ..Default::default()
        };
        user.merge_with(&other);
        assert!(user.allow_guest_calls);
    }

    #[test]
    fn test_merge_with_webrtc_flag() {
        let mut user = SipUser {
            username: "1001".to_string(),
            is_support_webrtc: false,
            ..Default::default()
        };
        let other = SipUser {
            username: "other".to_string(),
            is_support_webrtc: true,
            ..Default::default()
        };
        user.merge_with(&other);
        assert!(user.is_support_webrtc);
    }

    // ── forwarding_config ────────────────────────────────────────────────

    #[test]
    fn test_forwarding_config_always() {
        let user = SipUser {
            username: "1001".to_string(),
            call_forwarding_mode: Some("always".to_string()),
            call_forwarding_destination: Some("1002".to_string()),
            ..Default::default()
        };
        let cfg = user.forwarding_config().unwrap();
        assert_eq!(cfg.mode, CallForwardingMode::Always);
        assert_eq!(cfg.endpoint, TransferEndpoint::Uri("1002".to_string()));
        assert_eq!(cfg.timeout, Duration::from_secs(30)); // default
    }

    #[test]
    fn test_forwarding_config_when_busy() {
        let user = SipUser {
            username: "1001".to_string(),
            call_forwarding_mode: Some("when_busy".to_string()),
            call_forwarding_destination: Some("queue:support".to_string()),
            ..Default::default()
        };
        let cfg = user.forwarding_config().unwrap();
        assert_eq!(cfg.mode, CallForwardingMode::WhenBusy);
        assert_eq!(cfg.endpoint, TransferEndpoint::Queue("support".to_string()));
    }

    #[test]
    fn test_forwarding_config_busy_alias() {
        let user = SipUser {
            username: "1001".to_string(),
            call_forwarding_mode: Some("busy".to_string()),
            call_forwarding_destination: Some("1003".to_string()),
            ..Default::default()
        };
        let cfg = user.forwarding_config().unwrap();
        assert_eq!(cfg.mode, CallForwardingMode::WhenBusy);
    }

    #[test]
    fn test_forwarding_config_no_answer() {
        let user = SipUser {
            username: "1001".to_string(),
            call_forwarding_mode: Some("when_not_answered".to_string()),
            call_forwarding_destination: Some("1004".to_string()),
            call_forwarding_timeout: Some(45),
            ..Default::default()
        };
        let cfg = user.forwarding_config().unwrap();
        assert_eq!(cfg.mode, CallForwardingMode::WhenNoAnswer);
        assert_eq!(cfg.timeout, Duration::from_secs(45));
    }

    #[test]
    fn test_forwarding_config_no_answer_alias() {
        let user = SipUser {
            username: "1001".to_string(),
            call_forwarding_mode: Some("no_answer".to_string()),
            call_forwarding_destination: Some("1005".to_string()),
            ..Default::default()
        };
        let cfg = user.forwarding_config().unwrap();
        assert_eq!(cfg.mode, CallForwardingMode::WhenNoAnswer);
    }

    #[test]
    fn test_forwarding_config_none_mode_returns_none() {
        let user = SipUser {
            username: "1001".to_string(),
            call_forwarding_mode: Some("none".to_string()),
            call_forwarding_destination: Some("1002".to_string()),
            ..Default::default()
        };
        assert!(user.forwarding_config().is_none());
    }

    #[test]
    fn test_forwarding_config_empty_mode_returns_none() {
        let user = SipUser {
            username: "1001".to_string(),
            call_forwarding_mode: Some("".to_string()),
            call_forwarding_destination: Some("1002".to_string()),
            ..Default::default()
        };
        assert!(user.forwarding_config().is_none());
    }

    #[test]
    fn test_forwarding_config_no_destination_returns_none() {
        let user = SipUser {
            username: "1001".to_string(),
            call_forwarding_mode: Some("always".to_string()),
            ..Default::default()
        };
        assert!(user.forwarding_config().is_none());
    }

    #[test]
    fn test_forwarding_config_empty_destination_returns_none() {
        let user = SipUser {
            username: "1001".to_string(),
            call_forwarding_mode: Some("always".to_string()),
            call_forwarding_destination: Some("".to_string()),
            ..Default::default()
        };
        assert!(user.forwarding_config().is_none());
    }

    #[test]
    fn test_forwarding_config_invalid_mode_returns_none() {
        let user = SipUser {
            username: "1001".to_string(),
            call_forwarding_mode: Some("invalid_mode".to_string()),
            call_forwarding_destination: Some("1002".to_string()),
            ..Default::default()
        };
        assert!(user.forwarding_config().is_none());
    }

    #[test]
    fn test_forwarding_config_case_insensitive_mode() {
        let user = SipUser {
            username: "1001".to_string(),
            call_forwarding_mode: Some("ALWAYS".to_string()),
            call_forwarding_destination: Some("1002".to_string()),
            ..Default::default()
        };
        let cfg = user.forwarding_config().unwrap();
        assert_eq!(cfg.mode, CallForwardingMode::Always);
    }

    // ── auth_digest ──────────────────────────────────────────────────────

    fn md5_hex(input: &str) -> String {
        use md5::{Digest, Md5};
        let mut hasher = Md5::new();
        hasher.update(input);
        format!("{:x}", hasher.finalize())
    }

    #[test]
    fn test_auth_digest_md5() {
        let user = SipUser {
            username: "alice".to_string(),
            password: Some("secret".to_string()),
            realm: Some("example.com".to_string()),
            ..Default::default()
        };
        let digest = user.auth_digest(Algorithm::Md5);
        assert_eq!(digest, md5_hex("alice:example.com:secret"));
    }

    #[test]
    fn test_auth_digest_md5_no_password() {
        let user = SipUser {
            username: "bob".to_string(),
            realm: Some("test.com".to_string()),
            ..Default::default()
        };
        let digest = user.auth_digest(Algorithm::Md5);
        assert_eq!(digest, md5_hex("bob:test.com:"));
    }

    #[test]
    fn test_auth_digest_md5_no_realm() {
        let user = SipUser {
            username: "carol".to_string(),
            password: Some("pass".to_string()),
            ..Default::default()
        };
        let digest = user.auth_digest(Algorithm::Md5);
        assert_eq!(digest, md5_hex("carol::pass"));
    }

    #[test]
    fn test_auth_digest_sha256() {
        use sha2::{Digest, Sha256};
        let user = SipUser {
            username: "alice".to_string(),
            password: Some("secret".to_string()),
            realm: Some("example.com".to_string()),
            ..Default::default()
        };
        let digest = user.auth_digest(Algorithm::Sha256);
        let mut hasher = Sha256::new();
        hasher.update("alice:example.com:secret");
        let expected = format!("{:x}", hasher.finalize());
        assert_eq!(digest, expected);
    }

    #[test]
    fn test_auth_digest_sha512() {
        use sha2::{Digest, Sha512};
        let user = SipUser {
            username: "alice".to_string(),
            password: Some("secret".to_string()),
            realm: Some("example.com".to_string()),
            ..Default::default()
        };
        let digest = user.auth_digest(Algorithm::Sha512);
        let mut hasher = Sha512::new();
        hasher.update("alice:example.com:secret");
        let expected = format!("{:x}", hasher.finalize());
        assert_eq!(digest, expected);
    }

    #[test]
    fn test_auth_digest_md5_sess_same_as_md5() {
        let user = SipUser {
            username: "alice".to_string(),
            password: Some("secret".to_string()),
            realm: Some("example.com".to_string()),
            ..Default::default()
        };
        // Md5Sess uses the same HA1 calculation in our implementation
        let md5_digest = user.auth_digest(Algorithm::Md5);
        let md5sess_digest = user.auth_digest(Algorithm::Md5Sess);
        assert_eq!(md5_digest, md5sess_digest);
    }

    // ── SipCredential ────────────────────────────────────────────────────

    #[test]
    fn test_sip_credential_construction() {
        let cred = SipCredential {
            username: "user".to_string(),
            password: "pass".to_string(),
            realm: Some("example.com".to_string()),
        };
        assert_eq!(cred.username, "user");
        assert_eq!(cred.password, "pass");
        assert_eq!(cred.realm.as_deref(), Some("example.com"));
    }
}
