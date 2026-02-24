use crate::console::{ConsoleState, middleware::AuthRequired};
use crate::models::monitoring_event::{
    Column as MonitoringEventColumn, Entity as MonitoringEventEntity,
};
use axum::{
    Json, Router,
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::Utc;
use sea_orm::{EntityTrait, QueryOrder};
use sea_orm::sea_query::Order;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tracing::warn;

pub fn urls() -> Router<Arc<ConsoleState>> {
    Router::new()
        .route("/monitoring", get(page_monitoring))
        .route("/monitoring/active-calls", get(active_calls_json))
        .route("/monitoring/events", get(audit_events_json))
}

// ---------------------------------------------------------------------------
// Page handler
// ---------------------------------------------------------------------------

async fn page_monitoring(
    State(state): State<Arc<ConsoleState>>,
    AuthRequired(user): AuthRequired,
) -> Response {
    state.render(
        "console/monitoring.html",
        json!({
            "nav_active": "monitoring",
            "username": user.username,
            "email": user.email,
        }),
    )
}

// ---------------------------------------------------------------------------
// Active calls JSON endpoint (for polling)
// ---------------------------------------------------------------------------

async fn active_calls_json(
    State(state): State<Arc<ConsoleState>>,
    AuthRequired(_): AuthRequired,
) -> Response {
    let Some(server) = state.sip_server() else {
        return Json(json!({ "calls": [] })).into_response();
    };

    let entries = server.active_call_registry.list_recent(200);
    let now = Utc::now();

    let calls: Vec<serde_json::Value> = entries
        .into_iter()
        .map(|entry| {
            let duration_secs = if let Some(answered_at) = entry.answered_at {
                (now - answered_at).num_seconds().max(0)
            } else {
                (now - entry.started_at).num_seconds().max(0)
            };

            json!({
                "session_id": entry.session_id,
                "caller": entry.caller.unwrap_or_default(),
                "callee": entry.callee.unwrap_or_default(),
                "direction": entry.direction,
                "status": entry.status.to_string(),
                "duration_secs": duration_secs,
                "started_at": entry.started_at.to_rfc3339(),
                "answered_at": entry.answered_at.map(|t| t.to_rfc3339()),
            })
        })
        .collect();

    Json(json!({ "calls": calls })).into_response()
}

// ---------------------------------------------------------------------------
// Audit events JSON endpoint
// ---------------------------------------------------------------------------

#[derive(Default, Deserialize)]
struct AuditEventsQuery {
    #[serde(default = "default_event_limit")]
    limit: usize,
}

fn default_event_limit() -> usize {
    50
}

async fn audit_events_json(
    State(state): State<Arc<ConsoleState>>,
    AuthRequired(_): AuthRequired,
    Query(query): Query<AuditEventsQuery>,
) -> Response {
    let limit = query.limit.clamp(1, 200);
    let db = state.db();

    let events = match MonitoringEventEntity::find()
        .order_by(MonitoringEventColumn::Timestamp, Order::Desc)
        .all(db)
        .await
    {
        Ok(items) => {
            let items: Vec<_> = items.into_iter().take(limit).collect();
            items
        }
        Err(err) => {
            warn!("failed to list monitoring events: {}", err);
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": err.to_string()})),
            )
                .into_response();
        }
    };

    Json(json!({ "events": events })).into_response()
}
