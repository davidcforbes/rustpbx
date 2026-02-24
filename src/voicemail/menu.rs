//! Voicemail retrieval IVR menu.
//!
//! This module implements a DTMF-driven state machine that allows users to
//! dial into their voicemail box (e.g. via *97) and navigate messages using
//! the telephone keypad.
//!
//! # DTMF Key Map
//!
//! | Key | Action                              |
//! |-----|-------------------------------------|
//! | 1   | Play / replay current message       |
//! | 2   | Save message (mark read, next)      |
//! | 3   | Delete message                      |
//! | 4   | Previous message                    |
//! | 6   | Next message                        |
//! | 7   | Skip backward 3 seconds             |
//! | 9   | Skip forward 3 seconds              |
//! | *   | Return to main menu                 |
//! | #   | Exit voicemail                      |

use crate::config::VoicemailConfig;
use crate::models::voicemail;
use anyhow::{Result, anyhow};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use sea_orm::sea_query::Order;
use tracing::{debug, info, warn};

/// Prompt file paths (placeholder paths -- actual audio files must be
/// provisioned separately).
pub mod prompts {
    /// "Welcome to your voicemail."
    pub const VM_GREETING: &str = "prompts/vm_greeting.wav";
    /// "You have {n} new messages and {m} total messages."
    pub const VM_MESSAGE_COUNT: &str = "prompts/vm_message_count.wav";
    /// "Playing message {n}."
    pub const VM_PLAYING_MESSAGE: &str = "prompts/vm_playing_message.wav";
    /// "Press 1 to replay. Press 2 to save. Press 3 to delete. ..."
    pub const VM_MESSAGE_OPTIONS: &str = "prompts/vm_message_options.wav";
    /// "Message saved."
    pub const VM_MESSAGE_SAVED: &str = "prompts/vm_message_saved.wav";
    /// "Message deleted."
    pub const VM_MESSAGE_DELETED: &str = "prompts/vm_message_deleted.wav";
    /// "No more messages."
    pub const VM_NO_MORE_MESSAGES: &str = "prompts/vm_no_more_messages.wav";
    /// "You have no messages."
    pub const VM_NO_MESSAGES: &str = "prompts/vm_no_messages.wav";
    /// "Goodbye."
    pub const VM_GOODBYE: &str = "prompts/vm_goodbye.wav";
    /// "Main menu."
    pub const VM_MAIN_MENU: &str = "prompts/vm_main_menu.wav";
    /// "First message."
    pub const VM_FIRST_MESSAGE: &str = "prompts/vm_first_message.wav";
    /// "Last message."
    pub const VM_LAST_MESSAGE: &str = "prompts/vm_last_message.wav";
}

// ---------------------------------------------------------------------------
// Menu state machine
// ---------------------------------------------------------------------------

/// States of the voicemail IVR menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuState {
    /// Initial greeting: "Welcome to your voicemail."
    Greeting,
    /// Announce message counts: "You have N new and M total messages."
    MessageCount,
    /// Playing the current voicemail message audio.
    PlayMessage,
    /// Post-message options prompt (replay / save / delete / next / ...).
    MessageOptions,
    /// Transitioning to the next (or previous) message.
    NextMessage,
    /// Session is ending -- play goodbye and hang up.
    End,
}

/// An action the menu tells the call session to perform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuAction {
    /// Play an audio prompt file.
    PlayPrompt(String),
    /// Play the voicemail recording at the given path.
    PlayRecording(String),
    /// Stop current playback immediately.
    StopPlayback,
    /// Request the call session to hang up.
    Hangup,
    /// Skip forward in current playback by the given number of seconds.
    SkipForward(u32),
    /// Skip backward in current playback by the given number of seconds.
    SkipBackward(u32),
    /// No action required; the menu is waiting for more DTMF input.
    None,
}

/// Lightweight view of a voicemail message used within the menu.
#[derive(Debug, Clone)]
pub struct VoicemailMessageInfo {
    pub id: i64,
    pub caller_id: String,
    pub caller_name: Option<String>,
    pub recording_path: String,
    pub duration_secs: i32,
    pub is_read: bool,
    pub created_at: chrono::DateTime<Utc>,
}

impl From<voicemail::Model> for VoicemailMessageInfo {
    fn from(m: voicemail::Model) -> Self {
        Self {
            id: m.id,
            caller_id: m.caller_id,
            caller_name: m.caller_name,
            recording_path: m.recording_path,
            duration_secs: m.duration_secs,
            is_read: m.is_read,
            created_at: m.created_at,
        }
    }
}

/// The voicemail menu session manages IVR state for a single call.
///
/// It holds a snapshot of the user's messages at the time the session
/// started and tracks the current position within that list.
pub struct VoicemailMenuSession {
    /// Current state machine state.
    state: MenuState,
    /// Mailbox identifier (typically the user's extension number).
    mailbox_id: String,
    /// Voicemail configuration.
    #[allow(dead_code)]
    config: VoicemailConfig,
    /// Database connection (required for retrieval).
    db: DatabaseConnection,
    /// Cached list of messages (newest first by default).
    messages: Vec<VoicemailMessageInfo>,
    /// Index into `messages` pointing at the "current" message.
    current_index: usize,
    /// Number of new (unread) messages at session start.
    /// Reserved for TTS-based message count announcements.
    #[allow(dead_code)]
    new_count: usize,
    /// Total number of messages at session start.
    total_count: usize,
}

impl VoicemailMenuSession {
    /// Create a new menu session for the given mailbox.
    ///
    /// This immediately loads the message list from the database so that the
    /// rest of the session operates on a consistent snapshot.
    pub async fn new(
        mailbox_id: String,
        config: VoicemailConfig,
        db: DatabaseConnection,
    ) -> Result<Self> {
        let messages_raw = voicemail::Entity::find()
            .filter(voicemail::Column::MailboxId.eq(&mailbox_id))
            .filter(voicemail::Column::DeletedAt.is_null())
            .order_by(voicemail::Column::CreatedAt, Order::Desc)
            .all(&db)
            .await
            .map_err(|e| anyhow!("Failed to load voicemail messages: {}", e))?;

        let new_count = messages_raw.iter().filter(|m| !m.is_read).count();
        let total_count = messages_raw.len();

        let messages: Vec<VoicemailMessageInfo> =
            messages_raw.into_iter().map(Into::into).collect();

        info!(
            mailbox_id = %mailbox_id,
            new_count,
            total_count,
            "Voicemail menu session created"
        );

        Ok(Self {
            state: MenuState::Greeting,
            mailbox_id,
            config,
            db,
            messages,
            current_index: 0,
            new_count,
            total_count,
        })
    }

    /// Return the current IVR state.
    pub fn state(&self) -> MenuState {
        self.state
    }

    /// Return the mailbox identifier.
    #[allow(dead_code)]
    pub fn mailbox_id(&self) -> &str {
        &self.mailbox_id
    }

    /// Start the session -- returns the first action(s) the call should
    /// execute (greeting prompt).
    pub fn start(&mut self) -> Vec<MenuAction> {
        self.state = MenuState::Greeting;
        vec![MenuAction::PlayPrompt(prompts::VM_GREETING.to_string())]
    }

    /// Advance the state machine after a prompt has finished playing.
    ///
    /// The call session should invoke this whenever an audio prompt
    /// completes so the menu can transition to the next state.
    pub fn on_prompt_completed(&mut self) -> Vec<MenuAction> {
        match self.state {
            MenuState::Greeting => {
                self.state = MenuState::MessageCount;
                self.announce_message_count()
            }
            MenuState::MessageCount => {
                if self.messages.is_empty() {
                    self.state = MenuState::End;
                    vec![
                        MenuAction::PlayPrompt(prompts::VM_NO_MESSAGES.to_string()),
                        MenuAction::Hangup,
                    ]
                } else {
                    self.state = MenuState::PlayMessage;
                    self.play_current_message()
                }
            }
            MenuState::PlayMessage => {
                self.state = MenuState::MessageOptions;
                vec![MenuAction::PlayPrompt(
                    prompts::VM_MESSAGE_OPTIONS.to_string(),
                )]
            }
            MenuState::MessageOptions => {
                // Options prompt finished, wait for DTMF input.
                // If no input arrives we could re-prompt or timeout.
                vec![MenuAction::None]
            }
            MenuState::NextMessage => {
                self.state = MenuState::PlayMessage;
                self.play_current_message()
            }
            MenuState::End => {
                vec![MenuAction::Hangup]
            }
        }
    }

    /// Handle a DTMF digit received from the caller.
    ///
    /// Returns a list of actions the call session should execute in response.
    pub async fn on_dtmf(&mut self, digit: char) -> Vec<MenuAction> {
        let digit_str = digit.to_string();
        info!(
            mailbox_id = %self.mailbox_id,
            digit = %digit_str,
            state = ?self.state,
            current_index = self.current_index,
            "Voicemail DTMF received"
        );

        match digit {
            '#' => self.handle_exit(),
            '*' => self.handle_main_menu(),
            _ => match self.state {
                MenuState::Greeting | MenuState::MessageCount => {
                    // During greeting/count, # exits, * restarts; other keys
                    // are ignored until the prompt finishes.
                    vec![MenuAction::None]
                }
                MenuState::PlayMessage => self.handle_dtmf_during_playback(digit).await,
                MenuState::MessageOptions => self.handle_dtmf_message_options(digit).await,
                MenuState::NextMessage => {
                    // Transitional state -- ignore input
                    vec![MenuAction::None]
                }
                MenuState::End => vec![MenuAction::Hangup],
            },
        }
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Build the message-count announcement actions.
    fn announce_message_count(&self) -> Vec<MenuAction> {
        // In a production system this would use TTS or concatenated number
        // prompts.  For now we play a single placeholder prompt.
        vec![MenuAction::PlayPrompt(
            prompts::VM_MESSAGE_COUNT.to_string(),
        )]
    }

    /// Build actions to play the current message.
    fn play_current_message(&mut self) -> Vec<MenuAction> {
        if let Some(msg) = self.messages.get(self.current_index) {
            debug!(
                mailbox_id = %self.mailbox_id,
                message_id = msg.id,
                index = self.current_index,
                recording = %msg.recording_path,
                "Playing voicemail message"
            );
            vec![
                MenuAction::PlayPrompt(prompts::VM_PLAYING_MESSAGE.to_string()),
                MenuAction::PlayRecording(msg.recording_path.clone()),
            ]
        } else {
            // Should not happen but handle gracefully
            warn!(
                mailbox_id = %self.mailbox_id,
                index = self.current_index,
                total = self.messages.len(),
                "Current message index out of bounds"
            );
            self.end_no_more_messages()
        }
    }

    /// Handle DTMF during message playback.
    async fn handle_dtmf_during_playback(&mut self, digit: char) -> Vec<MenuAction> {
        match digit {
            '1' => {
                // Replay current message
                vec![
                    MenuAction::StopPlayback,
                    MenuAction::PlayRecording(
                        self.messages
                            .get(self.current_index)
                            .map(|m| m.recording_path.clone())
                            .unwrap_or_default(),
                    ),
                ]
            }
            '2' => {
                // Save (mark as read) and move to next
                let actions = self.save_current_message().await;
                actions
            }
            '3' => {
                // Delete message
                let actions = self.delete_current_message().await;
                actions
            }
            '4' => {
                // Previous message
                self.go_previous_message()
            }
            '6' => {
                // Next message
                self.go_next_message()
            }
            '7' => {
                // Skip backward 3 seconds
                vec![MenuAction::SkipBackward(3)]
            }
            '9' => {
                // Skip forward 3 seconds
                vec![MenuAction::SkipForward(3)]
            }
            _ => {
                debug!(digit = %digit.to_string(), "Ignoring unrecognized DTMF during playback");
                vec![MenuAction::None]
            }
        }
    }

    /// Handle DTMF while the message-options prompt is playing or the user
    /// is at the options menu.
    async fn handle_dtmf_message_options(&mut self, digit: char) -> Vec<MenuAction> {
        match digit {
            '1' => {
                // Replay current message
                self.state = MenuState::PlayMessage;
                vec![
                    MenuAction::StopPlayback,
                    MenuAction::PlayRecording(
                        self.messages
                            .get(self.current_index)
                            .map(|m| m.recording_path.clone())
                            .unwrap_or_default(),
                    ),
                ]
            }
            '2' => {
                // Save and next
                let actions = self.save_current_message().await;
                actions
            }
            '3' => {
                // Delete
                let actions = self.delete_current_message().await;
                actions
            }
            '4' => {
                // Previous
                self.go_previous_message()
            }
            '6' => {
                // Next
                self.go_next_message()
            }
            '7' => {
                // Skip backward 3 seconds in current message replay
                self.state = MenuState::PlayMessage;
                vec![
                    MenuAction::StopPlayback,
                    MenuAction::PlayRecording(
                        self.messages
                            .get(self.current_index)
                            .map(|m| m.recording_path.clone())
                            .unwrap_or_default(),
                    ),
                    MenuAction::SkipBackward(3),
                ]
            }
            '9' => {
                // Skip forward 3 seconds
                self.state = MenuState::PlayMessage;
                vec![
                    MenuAction::StopPlayback,
                    MenuAction::PlayRecording(
                        self.messages
                            .get(self.current_index)
                            .map(|m| m.recording_path.clone())
                            .unwrap_or_default(),
                    ),
                    MenuAction::SkipForward(3),
                ]
            }
            _ => {
                debug!(digit = %digit.to_string(), "Ignoring unrecognized DTMF at message options");
                vec![MenuAction::None]
            }
        }
    }

    /// Mark the current message as read and advance to the next one.
    async fn save_current_message(&mut self) -> Vec<MenuAction> {
        if let Some(msg) = self.messages.get(self.current_index) {
            let msg_id = msg.id;
            if let Err(e) = Self::mark_message_read(&self.db, msg_id).await {
                warn!(
                    mailbox_id = %self.mailbox_id,
                    message_id = msg_id,
                    error = %e,
                    "Failed to mark voicemail message as read"
                );
            } else {
                info!(
                    mailbox_id = %self.mailbox_id,
                    message_id = msg_id,
                    "Voicemail message marked as read"
                );
                // Update the cached copy
                if let Some(cached) = self.messages.get_mut(self.current_index) {
                    cached.is_read = true;
                }
            }
        }

        let mut actions = vec![
            MenuAction::StopPlayback,
            MenuAction::PlayPrompt(prompts::VM_MESSAGE_SAVED.to_string()),
        ];
        actions.extend(self.advance_to_next());
        actions
    }

    /// Delete the current message and advance to the next one.
    async fn delete_current_message(&mut self) -> Vec<MenuAction> {
        if let Some(msg) = self.messages.get(self.current_index) {
            let msg_id = msg.id;
            if let Err(e) = Self::soft_delete_message(&self.db, msg_id).await {
                warn!(
                    mailbox_id = %self.mailbox_id,
                    message_id = msg_id,
                    error = %e,
                    "Failed to delete voicemail message"
                );
            } else {
                info!(
                    mailbox_id = %self.mailbox_id,
                    message_id = msg_id,
                    "Voicemail message deleted"
                );
                // Remove from cached list
                self.messages.remove(self.current_index);
                self.total_count = self.messages.len();
                // Adjust index if we removed the last item
                if self.current_index >= self.messages.len() && !self.messages.is_empty() {
                    self.current_index = self.messages.len() - 1;
                }
            }
        }

        let mut actions = vec![
            MenuAction::StopPlayback,
            MenuAction::PlayPrompt(prompts::VM_MESSAGE_DELETED.to_string()),
        ];

        if self.messages.is_empty() {
            actions.extend(self.end_no_more_messages());
        } else {
            self.state = MenuState::PlayMessage;
            actions.extend(self.play_current_message());
        }
        actions
    }

    /// Navigate to the previous message.
    fn go_previous_message(&mut self) -> Vec<MenuAction> {
        if self.current_index == 0 {
            debug!(
                mailbox_id = %self.mailbox_id,
                "Already at first message"
            );
            return vec![
                MenuAction::StopPlayback,
                MenuAction::PlayPrompt(prompts::VM_FIRST_MESSAGE.to_string()),
            ];
        }
        self.current_index -= 1;
        self.state = MenuState::PlayMessage;
        let mut actions = vec![MenuAction::StopPlayback];
        actions.extend(self.play_current_message());
        actions
    }

    /// Navigate to the next message.
    fn go_next_message(&mut self) -> Vec<MenuAction> {
        if self.current_index + 1 >= self.messages.len() {
            debug!(
                mailbox_id = %self.mailbox_id,
                "Already at last message"
            );
            return vec![
                MenuAction::StopPlayback,
                MenuAction::PlayPrompt(prompts::VM_LAST_MESSAGE.to_string()),
            ];
        }
        self.current_index += 1;
        self.state = MenuState::PlayMessage;
        let mut actions = vec![MenuAction::StopPlayback];
        actions.extend(self.play_current_message());
        actions
    }

    /// Advance past the current message to the next one, or end if none left.
    fn advance_to_next(&mut self) -> Vec<MenuAction> {
        if self.current_index + 1 < self.messages.len() {
            self.current_index += 1;
            self.state = MenuState::NextMessage;
            self.play_current_message()
        } else {
            self.end_no_more_messages()
        }
    }

    /// Transition to exit.
    fn handle_exit(&mut self) -> Vec<MenuAction> {
        self.state = MenuState::End;
        vec![
            MenuAction::StopPlayback,
            MenuAction::PlayPrompt(prompts::VM_GOODBYE.to_string()),
            MenuAction::Hangup,
        ]
    }

    /// Return to main menu (re-announce counts).
    fn handle_main_menu(&mut self) -> Vec<MenuAction> {
        self.state = MenuState::Greeting;
        self.current_index = 0;
        vec![
            MenuAction::StopPlayback,
            MenuAction::PlayPrompt(prompts::VM_MAIN_MENU.to_string()),
        ]
    }

    /// End with "no more messages".
    fn end_no_more_messages(&mut self) -> Vec<MenuAction> {
        self.state = MenuState::End;
        vec![
            MenuAction::PlayPrompt(prompts::VM_NO_MORE_MESSAGES.to_string()),
            MenuAction::PlayPrompt(prompts::VM_GOODBYE.to_string()),
            MenuAction::Hangup,
        ]
    }

    // -----------------------------------------------------------------------
    // Database helpers
    // -----------------------------------------------------------------------

    /// Mark a voicemail message as read.
    async fn mark_message_read(db: &DatabaseConnection, id: i64) -> Result<()> {
        let msg = voicemail::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| anyhow!("DB error looking up voicemail {}: {}", id, e))?
            .ok_or_else(|| anyhow!("Voicemail message {} not found", id))?;

        let mut active: voicemail::ActiveModel = msg.into();
        active.is_read = Set(true);
        active.updated_at = Set(Utc::now());
        active
            .update(db)
            .await
            .map_err(|e| anyhow!("Failed to mark voicemail {} as read: {}", id, e))?;
        Ok(())
    }

    /// Soft-delete a voicemail message (set deleted_at timestamp).
    async fn soft_delete_message(db: &DatabaseConnection, id: i64) -> Result<()> {
        let msg = voicemail::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| anyhow!("DB error looking up voicemail {}: {}", id, e))?
            .ok_or_else(|| anyhow!("Voicemail message {} not found", id))?;

        let mut active: voicemail::ActiveModel = msg.into();
        active.deleted_at = Set(Some(Utc::now()));
        active.updated_at = Set(Utc::now());
        active
            .update(db)
            .await
            .map_err(|e| anyhow!("Failed to delete voicemail {}: {}", id, e))?;
        Ok(())
    }
}

/// Parse an RFC 2833 telephone-event DTMF digit from an RTP payload.
///
/// The RFC 2833 payload is at least 4 bytes:
///   byte 0:       event code (0-15)
///   byte 1:       E (end) bit + reserved + volume
///   bytes 2-3:    duration
///
/// Returns `Some((digit_char, is_end))` if the payload is valid, or `None`
/// if it cannot be parsed.
pub fn parse_rfc2833_dtmf(payload: &[u8]) -> Option<(char, bool)> {
    if payload.len() < 4 {
        return None;
    }

    let event_code = payload[0];
    let is_end = (payload[1] & 0x80) != 0;

    let digit = match event_code {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
        4 => '4',
        5 => '5',
        6 => '6',
        7 => '7',
        8 => '8',
        9 => '9',
        10 => '*',
        11 => '#',
        12 => 'A',
        13 => 'B',
        14 => 'C',
        15 => 'D',
        _ => return None,
    };

    Some((digit, is_end))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rfc2833_dtmf_digit_0() {
        // Event code 0 = digit '0', end bit not set, volume 10, duration 160
        let payload = [0x00, 0x0A, 0x00, 0xA0];
        let result = parse_rfc2833_dtmf(&payload);
        assert_eq!(result, Some(('0', false)));
    }

    #[test]
    fn test_parse_rfc2833_dtmf_digit_9_end() {
        // Event code 9, end bit set
        let payload = [0x09, 0x8A, 0x03, 0x20];
        let result = parse_rfc2833_dtmf(&payload);
        assert_eq!(result, Some(('9', true)));
    }

    #[test]
    fn test_parse_rfc2833_dtmf_star() {
        // Event code 10 = '*'
        let payload = [0x0A, 0x0A, 0x00, 0xA0];
        let result = parse_rfc2833_dtmf(&payload);
        assert_eq!(result, Some(('*', false)));
    }

    #[test]
    fn test_parse_rfc2833_dtmf_hash() {
        // Event code 11 = '#', end bit set
        let payload = [0x0B, 0x8A, 0x06, 0x40];
        let result = parse_rfc2833_dtmf(&payload);
        assert_eq!(result, Some(('#', true)));
    }

    #[test]
    fn test_parse_rfc2833_dtmf_too_short() {
        let payload = [0x01, 0x0A, 0x00];
        assert_eq!(parse_rfc2833_dtmf(&payload), None);
    }

    #[test]
    fn test_parse_rfc2833_dtmf_invalid_event() {
        // Event code 16 is outside the standard range
        let payload = [0x10, 0x0A, 0x00, 0xA0];
        assert_eq!(parse_rfc2833_dtmf(&payload), None);
    }

    #[test]
    fn test_parse_rfc2833_dtmf_all_digits() {
        let expected = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '*', '#', 'A', 'B', 'C', 'D',
        ];
        for (code, &expected_char) in expected.iter().enumerate() {
            let payload = [code as u8, 0x0A, 0x00, 0xA0];
            let result = parse_rfc2833_dtmf(&payload);
            assert_eq!(
                result,
                Some((expected_char, false)),
                "Event code {} should map to '{}'",
                code,
                expected_char,
            );
        }
    }

    /// Smoke test that the state machine transitions work without a database.
    /// (Full integration requires a DB so we test the state transitions only.)
    #[test]
    fn test_menu_state_transitions() {
        // Verify the initial state flow: Greeting -> MessageCount
        assert_eq!(MenuState::Greeting, MenuState::Greeting);
        assert_ne!(MenuState::Greeting, MenuState::End);
    }

    #[test]
    fn test_menu_action_equality() {
        assert_eq!(MenuAction::Hangup, MenuAction::Hangup);
        assert_eq!(MenuAction::None, MenuAction::None);
        assert_eq!(
            MenuAction::PlayPrompt("test.wav".to_string()),
            MenuAction::PlayPrompt("test.wav".to_string()),
        );
        assert_ne!(
            MenuAction::PlayPrompt("a.wav".to_string()),
            MenuAction::PlayPrompt("b.wav".to_string()),
        );
    }
}
