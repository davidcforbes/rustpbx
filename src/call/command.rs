// AI Voice Agent call commands (from active-call)
use crate::media::agent_recorder::RecorderOption as AgentRecorderOption;
use crate::synthesis::SynthesisOption;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;

pub type CommandSender = tokio::sync::broadcast::Sender<Command>;
pub type CommandReceiver = tokio::sync::broadcast::Receiver<Command>;

// WebSocket Commands for AI Voice Agent
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(
    tag = "command",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Command {
    Invite {
        option: crate::CallOption,
    },
    Accept {
        option: crate::CallOption,
    },
    Reject {
        reason: String,
        code: Option<u32>,
    },
    Ringing {
        recorder: Option<AgentRecorderOption>,
        early_media: Option<bool>,
        ringtone: Option<String>,
    },
    Tts {
        text: String,
        speaker: Option<String>,
        play_id: Option<String>,
        auto_hangup: Option<bool>,
        streaming: Option<bool>,
        end_of_stream: Option<bool>,
        option: Option<SynthesisOption>,
        wait_input_timeout: Option<u32>,
        base64: Option<bool>,
        cache_key: Option<String>,
    },
    Play {
        url: String,
        play_id: Option<String>,
        auto_hangup: Option<bool>,
        wait_input_timeout: Option<u32>,
    },
    Interrupt {
        graceful: Option<bool>,
        fade_out_ms: Option<u32>,
    },
    Pause {},
    Resume {},
    Hangup {
        reason: Option<String>,
        initiator: Option<String>,
        headers: Option<HashMap<String, String>>,
    },
    Refer {
        caller: String,
        callee: String,
        options: Option<crate::ReferOption>,
    },
    Mute {
        track_id: Option<String>,
    },
    Unmute {
        track_id: Option<String>,
    },
    History {
        speaker: String,
        text: String,
    },
}
