use crate::app::AppState;
use axum::{
    Json, Router,
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCallRequest {
    pub playbook: Option<String>,
    pub option: Option<crate::CallOption>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallInfo {
    pub session_id: String,
    pub status: String,
}

/// POST /voice-agent/v1/calls - Create a new voice agent call
async fn create_call(
    State(_state): State<AppState>,
    Json(_req): Json<CreateCallRequest>,
) -> impl IntoResponse {
    // TODO: Create ActiveCall with playbook
    let session_id = uuid::Uuid::new_v4().to_string();
    info!(session_id, "voice-agent: create call request");

    (
        StatusCode::CREATED,
        Json(CallInfo {
            session_id,
            status: "created".to_string(),
        }),
    )
}

/// GET /voice-agent/v1/calls - List active voice agent calls
#[cfg(feature = "voice-agent")]
async fn list_calls(State(state): State<AppState>) -> impl IntoResponse {
    let calls: Vec<CallInfo> = {
        let active = state.active_calls.lock().unwrap();
        active
            .keys()
            .map(|id| CallInfo {
                session_id: id.clone(),
                status: "active".to_string(),
            })
            .collect()
    };
    Json(calls)
}

#[cfg(not(feature = "voice-agent"))]
async fn list_calls() -> impl IntoResponse {
    Json(Vec::<CallInfo>::new())
}

/// DELETE /voice-agent/v1/calls/:session_id - Hangup a call
#[cfg(feature = "voice-agent")]
async fn hangup_call(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let call = {
        let active = state.active_calls.lock().unwrap();
        active.get(&session_id).cloned()
    };
    match call {
        Some(_) => {
            info!(session_id, "voice-agent: hangup request");
            StatusCode::OK
        }
        None => StatusCode::NOT_FOUND,
    }
}

#[cfg(not(feature = "voice-agent"))]
async fn hangup_call(Path(_session_id): Path<String>) -> impl IntoResponse {
    StatusCode::NOT_FOUND
}

/// POST /voice-agent/v1/calls/:session_id/command - Send command to active call
async fn send_command(
    State(_state): State<AppState>,
    Path(session_id): Path<String>,
    Json(_cmd): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!(session_id, "voice-agent: command received");
    // TODO: Forward command to ActiveCall via cmd_sender
    StatusCode::ACCEPTED
}

/// GET /voice-agent/v1/ws - WebSocket endpoint for voice agent
async fn ws_handler(ws: WebSocketUpgrade, State(_state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        info!("voice-agent: WebSocket connected");
        // TODO: Handle WebSocket audio/control messages
        let _ = socket;
    })
}

pub fn voice_agent_router() -> Router<AppState> {
    Router::new()
        .route("/calls", post(create_call).get(list_calls))
        .route("/calls/{session_id}", delete(hangup_call))
        .route("/calls/{session_id}/command", post(send_command))
        .route("/ws", get(ws_handler))
}
