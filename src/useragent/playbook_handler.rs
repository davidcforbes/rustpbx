use crate::{
    app::AppState, call::RoutingState, config::PlaybookRule,
    useragent::invitation::InvitationHandler,
};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use regex::Regex;
use rsip::prelude::HeadersExt;
use rsipstack::dialog::server_dialog::ServerInviteDialog;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

pub struct PlaybookInvitationHandler {
    rules: Vec<CompiledPlaybookRule>,
    default: Option<String>,
    app_state: AppState,
}

struct CompiledPlaybookRule {
    caller: Option<Regex>,
    callee: Option<Regex>,
    playbook: String,
}

impl PlaybookInvitationHandler {
    pub fn new(
        rules: Vec<PlaybookRule>,
        default: Option<String>,
        app_state: AppState,
    ) -> Result<Self> {
        let mut compiled_rules = Vec::new();

        for rule in rules {
            let caller_regex = if let Some(pattern) = rule.caller {
                Some(
                    Regex::new(&pattern)
                        .map_err(|e| anyhow!("invalid caller regex '{}': {}", pattern, e))?,
                )
            } else {
                None
            };

            let callee_regex = if let Some(pattern) = rule.callee {
                Some(
                    Regex::new(&pattern)
                        .map_err(|e| anyhow!("invalid callee regex '{}': {}", pattern, e))?,
                )
            } else {
                None
            };

            compiled_rules.push(CompiledPlaybookRule {
                caller: caller_regex,
                callee: callee_regex,
                playbook: rule.playbook.clone(),
            });
        }

        Ok(Self {
            rules: compiled_rules,
            default,
            app_state,
        })
    }

    pub fn match_playbook(&self, caller: &str, callee: &str) -> Option<String> {
        for rule in &self.rules {
            let caller_matches = rule
                .caller
                .as_ref()
                .map(|r| r.is_match(caller))
                .unwrap_or(true);

            let callee_matches = rule
                .callee
                .as_ref()
                .map(|r| r.is_match(callee))
                .unwrap_or(true);

            if caller_matches && callee_matches {
                return Some(rule.playbook.clone());
            }
        }

        self.default.clone()
    }

    fn extract_custom_headers(
        headers: &rsip::Headers,
    ) -> std::collections::HashMap<String, serde_json::Value> {
        let mut extras = std::collections::HashMap::new();
        for header in headers.iter() {
            if let rsip::Header::Other(name, value) = header {
                // Capture all custom headers, Playbook logic can filter them later using `sip.extract_headers` if needed
                extras.insert(
                    name.to_string(),
                    serde_json::Value::String(value.to_string()),
                );
            }
        }
        extras
    }
}

#[async_trait]
impl InvitationHandler for PlaybookInvitationHandler {
    async fn on_invite(
        &self,
        dialog_id: String,
        cancel_token: CancellationToken,
        dialog: ServerInviteDialog,
        _routing_state: Arc<RoutingState>,
    ) -> Result<()> {
        let invite_request = dialog.initial_request();
        let caller = invite_request.from_header()?.uri()?.to_string();
        let callee = invite_request.to_header()?.uri()?.to_string();

        match self.match_playbook(&caller, &callee) {
            Some(playbook) => {
                info!(
                    dialog_id,
                    caller, callee, playbook, "matched playbook for invite"
                );

                // Extract custom headers
                let mut extras = Self::extract_custom_headers(&invite_request.headers);

                // Inject built-in caller/callee variables
                extras.insert(
                    crate::playbook::BUILTIN_CALLER.to_string(),
                    serde_json::Value::String(caller.clone()),
                );
                extras.insert(
                    crate::playbook::BUILTIN_CALLEE.to_string(),
                    serde_json::Value::String(callee.clone()),
                );

                if !extras.is_empty() {
                    let mut params = self.app_state.pending_params.lock().await;
                    params.insert(dialog_id.clone(), extras);
                }

                // Store the playbook name in pending_playbooks
                {
                    let mut pending = self.app_state.pending_playbooks.lock().await;
                    pending.insert(dialog_id.clone(), playbook);
                }

                // Start call handler in background task
                let app_state = self.app_state.clone();
                let session_id = dialog_id.clone();
                let cancel_token_clone = cancel_token.clone();

                crate::spawn(async move {
                    use crate::call::ActiveCallType;
                    use crate::call::active_call::{ActiveCall, ActiveCallGuard};
                    use crate::media::track::TrackConfig;
                    use crate::playbook::Playbook;
                    use crate::playbook::runner::PlaybookRunner;
                    use std::path::PathBuf;

                    // Retrieve the playbook name from pending
                    let playbook_name = {
                        let mut pending = app_state.pending_playbooks.lock().await;
                        pending.remove(&session_id)
                    };

                    let name_or_content = match playbook_name {
                        Some(n) => n,
                        None => {
                            warn!(session_id, "No playbook found in pending, rejecting");
                            if let Err(e) = dialog.reject(
                                Some(rsip::StatusCode::ServiceUnavailable),
                                Some("No Playbook".to_string()),
                            ) {
                                warn!(session_id, "Failed to reject SIP dialog: {}", e);
                            }
                            return;
                        }
                    };

                    // Load the playbook (from file or inline YAML)
                    let playbook = if name_or_content.trim().starts_with("---") {
                        // Inline YAML content
                        match Playbook::parse(&name_or_content) {
                            Ok(pb) => pb,
                            Err(e) => {
                                warn!(session_id, "Failed to parse inline playbook: {}", e);
                                if let Err(e) = dialog.reject(
                                    Some(rsip::StatusCode::ServiceUnavailable),
                                    Some("Invalid Playbook".to_string()),
                                ) {
                                    warn!(session_id, "Failed to reject SIP dialog: {}", e);
                                }
                                return;
                            }
                        }
                    } else {
                        // File path
                        let path = if name_or_content.starts_with("config/playbook/") {
                            PathBuf::from(&name_or_content)
                        } else {
                            PathBuf::from("config/playbook").join(&name_or_content)
                        };

                        if !path.exists() {
                            warn!(session_id, path=?path, "Playbook file not found, rejecting SIP call");
                            if let Err(e) = dialog.reject(
                                Some(rsip::StatusCode::ServiceUnavailable),
                                Some("Playbook Not Found".to_string()),
                            ) {
                                warn!(session_id, "Failed to reject SIP dialog: {}", e);
                            }
                            return;
                        }

                        match Playbook::load(&path).await {
                            Ok(pb) => pb,
                            Err(e) => {
                                warn!(session_id, path=?path, "Failed to load playbook: {}", e);
                                if let Err(e) = dialog.reject(
                                    Some(rsip::StatusCode::ServiceUnavailable),
                                    Some("Playbook Load Error".to_string()),
                                ) {
                                    warn!(session_id, "Failed to reject SIP dialog: {}", e);
                                }
                                return;
                            }
                        }
                    };

                    // Retrieve extras (custom SIP headers, caller, callee) from pending_params
                    let extras = {
                        let mut params = app_state.pending_params.lock().await;
                        params.remove(&session_id)
                    };

                    // Process extract_headers from playbook's SIP config
                    let extras = if let Some(ref sip_config) = playbook.config.sip {
                        if let Some(ref extract_list) = sip_config.extract_headers {
                            let mut e = extras.unwrap_or_default();
                            let sip_header_keys: Vec<String> = extract_list.clone();
                            e.insert(
                                "_sip_header_keys".to_string(),
                                serde_json::to_value(&sip_header_keys).unwrap_or_default(),
                            );
                            Some(e)
                        } else {
                            extras
                        }
                    } else {
                        extras
                    };

                    // Render playbook templates with variables
                    let playbook = if let Some(ref vars) = extras {
                        match playbook.render(vars) {
                            Ok(rendered) => rendered,
                            Err(e) => {
                                warn!(session_id, "Failed to render playbook: {}", e);
                                playbook
                            }
                        }
                    } else {
                        playbook
                    };

                    // Get hangup headers template from SIP config
                    let sip_hangup_headers = playbook.config.sip.as_ref()
                        .and_then(|s| s.hangup_headers.clone());

                    // Create ActiveCall
                    let track_config = TrackConfig::default();
                    let active_call = std::sync::Arc::new(ActiveCall::new(
                        ActiveCallType::Sip,
                        cancel_token_clone.clone(),
                        session_id.clone(),
                        app_state.invitation.clone(),
                        app_state.clone(),
                        track_config,
                        None, // no websocket audio
                        false, // dump_events
                        None, // server_side_track_id
                        extras,
                        sip_hangup_headers,
                    ));

                    // Register as pending dialog so ActiveCall can accept the SIP invite
                    let dialog_id_parsed = dialog.id();
                    let (_, state_receiver) = tokio::sync::mpsc::unbounded_channel();
                    let pending = crate::useragent::invitation::PendingDialog {
                        token: cancel_token_clone.clone(),
                        dialog,
                        state_receiver,
                    };
                    app_state.invitation.add_pending(dialog_id_parsed, pending);

                    // Create receiver BEFORE serve() (broadcast doesn't cache)
                    let receiver = active_call.new_receiver();

                    // Create ActiveCallGuard to track the call
                    let _guard = ActiveCallGuard::new(active_call.clone());

                    // Send Accept command to trigger SDP negotiation
                    if let Err(e) = active_call.enqueue_command(crate::call::Command::Accept {
                        option: Default::default(),
                    }).await {
                        warn!(session_id, "Failed to send accept command: {}", e);
                        return;
                    }

                    // Create PlaybookRunner
                    let runner = match PlaybookRunner::new(playbook, active_call.clone()) {
                        Ok(r) => r,
                        Err(e) => {
                            warn!(session_id, "Failed to create PlaybookRunner: {}", e);
                            cancel_token_clone.cancel();
                            return;
                        }
                    };

                    info!(session_id, "Starting voice agent call handler");

                    // Run ActiveCall::serve() and PlaybookRunner in parallel
                    tokio::select! {
                        result = active_call.serve(receiver) => {
                            if let Err(e) = result {
                                warn!(session_id, "ActiveCall serve error: {}", e);
                            }
                            info!(session_id, "SIP call handler completed");
                        }
                        _ = runner.run() => {
                            info!(session_id, "PlaybookRunner completed");
                        }
                        _ = cancel_token_clone.cancelled() => {
                            info!(session_id, "SIP call cancelled");
                        }
                    }
                });

                Ok(())
            }
            None => {
                warn!(
                    dialog_id,
                    caller, callee, "no playbook matched for invite, rejecting"
                );
                Err(anyhow!(
                    "no matching playbook found for caller {} and callee {}",
                    caller,
                    callee
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PlaybookRule;

    // Simpler helper that creates just the matching function for testing
    struct TestMatcher {
        rules: Vec<(Option<Regex>, Option<Regex>, String)>,
        default: Option<String>,
    }

    impl TestMatcher {
        fn new(rules: Vec<PlaybookRule>, default: Option<String>) -> Result<Self> {
            let mut compiled_rules = Vec::new();

            for rule in rules {
                let caller_regex = if let Some(pattern) = rule.caller {
                    Some(
                        Regex::new(&pattern)
                            .map_err(|e| anyhow!("invalid caller regex '{}': {}", pattern, e))?,
                    )
                } else {
                    None
                };

                let callee_regex = if let Some(pattern) = rule.callee {
                    Some(
                        Regex::new(&pattern)
                            .map_err(|e| anyhow!("invalid callee regex '{}': {}", pattern, e))?,
                    )
                } else {
                    None
                };

                compiled_rules.push((caller_regex, callee_regex, rule.playbook.clone()));
            }

            Ok(Self {
                rules: compiled_rules,
                default,
            })
        }

        fn match_playbook(&self, caller: &str, callee: &str) -> Option<String> {
            for (caller_re, callee_re, playbook) in &self.rules {
                let caller_matches = caller_re
                    .as_ref()
                    .map(|r| r.is_match(caller))
                    .unwrap_or(true);

                let callee_matches = callee_re
                    .as_ref()
                    .map(|r| r.is_match(callee))
                    .unwrap_or(true);

                if caller_matches && callee_matches {
                    return Some(playbook.clone());
                }
            }

            self.default.clone()
        }
    }

    #[test]
    fn test_playbook_rule_matching() {
        let rules = vec![
            PlaybookRule {
                caller: Some(r"^\+1\d{10}$".to_string()),
                callee: Some(r"^sip:support@.*".to_string()),
                playbook: "support.md".to_string(),
            },
            PlaybookRule {
                caller: Some(r"^\+86\d+$".to_string()),
                callee: None,
                playbook: "chinese.md".to_string(),
            },
            PlaybookRule {
                caller: None,
                callee: Some(r"^sip:sales@.*".to_string()),
                playbook: "sales.md".to_string(),
            },
        ];

        let matcher = TestMatcher::new(rules, Some("default.md".to_string())).unwrap();

        // Test US number to support
        assert_eq!(
            matcher.match_playbook("+12125551234", "sip:support@example.com"),
            Some("support.md".to_string())
        );

        // Test Chinese number (matches second rule)
        assert_eq!(
            matcher.match_playbook("+8613800138000", "sip:any@example.com"),
            Some("chinese.md".to_string())
        );

        // Test sales callee (matches third rule)
        assert_eq!(
            matcher.match_playbook("+44123456789", "sip:sales@example.com"),
            Some("sales.md".to_string())
        );

        // Test no match - should use default
        assert_eq!(
            matcher.match_playbook("+44123456789", "sip:other@example.com"),
            Some("default.md".to_string())
        );
    }

    #[test]
    fn test_playbook_rule_no_default() {
        let rules = vec![PlaybookRule {
            caller: Some(r"^\+1.*".to_string()),
            callee: None,
            playbook: "us.md".to_string(),
        }];

        let matcher = TestMatcher::new(rules, None).unwrap();

        // Matches
        assert_eq!(
            matcher.match_playbook("+12125551234", "sip:any@example.com"),
            Some("us.md".to_string())
        );

        // No match and no default
        assert_eq!(
            matcher.match_playbook("+44123456789", "sip:any@example.com"),
            None
        );
    }

    #[test]
    fn test_invalid_regex() {
        let rules = vec![PlaybookRule {
            caller: Some(r"[invalid(".to_string()),
            callee: None,
            playbook: "test.md".to_string(),
        }];

        let result = TestMatcher::new(rules, None);
        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("invalid caller regex"));
    }

    #[test]
    fn test_extract_custom_headers() {
        use rsip::Header;

        let mut headers = rsip::Headers::default();
        headers.push(Header::ContentLength(10.into())); // Standard header (ignored)
        headers.push(Header::Other("X-Tenant-ID".into(), "123".into()));
        headers.push(Header::Other("Custom-Header".into(), "xyz".into()));

        let extras = PlaybookInvitationHandler::extract_custom_headers(&headers);

        assert_eq!(extras.len(), 2);
        assert_eq!(
            extras.get("X-Tenant-ID").unwrap(),
            &serde_json::Value::String("123".to_string())
        );
        assert_eq!(
            extras.get("Custom-Header").unwrap(),
            &serde_json::Value::String("xyz".to_string())
        );
    }
}
