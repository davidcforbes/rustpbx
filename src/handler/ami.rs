use crate::{
    app::AppState,
    config::{Config, ProxyConfig},
    handler::middleware::clientaddr::ClientAddr,
    preflight,
    proxy::proxy_call::media_bridge::MonitorMode,
    proxy::proxy_call::state::SessionAction,
};
use axum::{
    Json, Router,
    extract::{FromRequest, Path, Query, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use serde::Deserialize;
use std::sync::{Arc, atomic::Ordering};
use tokio::time::{Duration, sleep};
use tracing::{info, warn};

pub fn ami_router(app_state: AppState) -> Router<AppState> {
    let r = Router::new()
        .route("/health", get(health_handler))
        .route("/dialogs", get(list_dialogs))
        .route("/hangup/{id}", get(hangup_dialog))
        .route("/transactions", get(list_transactions))
        .route("/shutdown", post(shutdown_handler))
        .route("/reload/trunks", post(reload_trunks_handler))
        .route("/reload/routes", post(reload_routes_handler))
        .route("/reload/acl", post(reload_acl_handler))
        .route("/reload/app", post(reload_app_handler))
        .route("/backup/status", get(backup_status_handler))
        .route("/backup/trigger", post(backup_trigger_handler))
        .route("/backup/health", get(backup_health_handler))
        .route("/backup/verify", get(backup_verify_handler))
        .route("/backup/history", get(backup_history_handler))
        .route(
            "/frequency_limits",
            get(list_frequency_limits).delete(clear_frequency_limits),
        )
        .route("/calls/{session_id}/monitor/start", post(monitor_start_handler))
        .route("/calls/{session_id}/monitor/stop", post(monitor_stop_handler))
        .route("/calls/{session_id}/monitor/mode", post(monitor_set_mode_handler))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            crate::handler::middleware::ami_auth::ami_auth_middleware,
        ));
    Router::new().nest("/ami/v1", r).with_state(app_state)
}

pub(super) async fn health_handler(State(state): State<AppState>) -> Response {
    let tx_stats = state.sip_server().inner.endpoint.inner.get_stats();
    let app_tasks = {
        let metrics = crate::utils::GLOBAL_TASK_METRICS.lock().unwrap();
        metrics
            .iter()
            .filter(|&(_, &v)| v > 0)
            .map(|(k, &v)| (k.clone(), serde_json::json!(v)))
            .collect::<serde_json::Map<String, serde_json::Value>>()
    };

    let sipserver_stats = serde_json::json!({
        "transactions": serde_json::json!({
            "running": tx_stats.running_transactions,
            "finished": tx_stats.finished_transactions,
            "waiting_ack": tx_stats.waiting_ack,
        }),
        "dialogs": state.sip_server().inner.dialog_layer.len(),
        "calls": state.sip_server().inner.active_call_registry.count(),
        "running_tx": state.sip_server().inner.runnings_tx.load(Ordering::Relaxed),
    });

    let callrecord_stats = match state.core.callrecord_stats {
        Some(ref stats) => serde_json::json!(stats.as_ref() as &crate::callrecord::CallRecordStats),
        None => {
            serde_json::json!({})
        }
    };

    let health = serde_json::json!({
        "status": "running",
        "uptime": state.uptime,
        "version": crate::version::get_version_info(),
        "total": state.total_calls.load(Ordering::Relaxed),
        "failed": state.total_failed_calls.load(Ordering::Relaxed),
        "tasks": app_tasks,
        "sipserver": sipserver_stats,
        "callrecord": callrecord_stats,
    });
    Json(health).into_response()
}

async fn shutdown_handler(State(state): State<AppState>, client_ip: ClientAddr) -> Response {
    warn!(%client_ip, "Shutdown initiated via /shutdown endpoint");
    state.token().cancel();
    Json(serde_json::json!({"status": "shutdown initiated"})).into_response()
}

trait DialogInfo {
    fn to_json(&self) -> serde_json::Value;
}

impl DialogInfo for rsipstack::dialog::dialog::Dialog {
    fn to_json(&self) -> serde_json::Value {
        let state = self.state();
        serde_json::json!({
            "state": state.to_string(),
            "from": self.from().to_string(),
            "to": self.to().to_string()
        })
    }
}

async fn list_dialogs(State(state): State<AppState>) -> Response {
    let mut result = Vec::new();
    let ids = state.sip_server().inner.dialog_layer.all_dialog_ids();
    for id in ids {
        if let Some(dialog) = state.sip_server().inner.dialog_layer.get_dialog_with(&id) {
            result.push(dialog.to_json());
        }
    }
    Json(result).into_response()
}

async fn hangup_dialog(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    match state.sip_server.inner.dialog_layer.get_dialog_with(&id) {
        Some(dlg) => match dlg.hangup().await {
            Ok(()) => {
                return Json(serde_json::json!({
                    "status": "ok",
                    "message": format!("Dialog with id '{}' hangup initiated", id),
                }))
                .into_response();
            }
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "status": "error",
                        "message": format!("Failed to hangup dialog with id '{}': {}", id, err),
                    })),
                )
                    .into_response();
            }
        },
        None => {}
    }
    return (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "status": "error",
            "message": format!("Dialog with id '{}' not found", id),
        })),
    )
        .into_response();
}

async fn list_transactions(State(state): State<AppState>) -> Response {
    let mut result = Vec::new();
    state
        .sip_server()
        .inner
        .endpoint
        .inner
        .get_running_transactions()
        .map(|ids| result.extend(ids));
    let result: Vec<String> = result.iter().map(|key| key.to_string()).collect();
    Json(result).into_response()
}

async fn reload_trunks_handler(State(state): State<AppState>, client_ip: ClientAddr) -> Response {
    info!(%client_ip, "Reload SIP trunks via /reload/trunks endpoint");

    let config_override = match load_proxy_config_override(&state) {
        Ok(cfg) => cfg,
        Err(response) => return response,
    };

    match state
        .sip_server()
        .inner
        .data_context
        .reload_trunks(true, config_override)
        .await
    {
        Ok(metrics) => {
            let total = metrics.total;
            Json(serde_json::json!({
                "status": "ok",
                "trunks_reloaded": total,
                "metrics": metrics,
            }))
        }
        .into_response(),
        Err(error) => {
            warn!(%client_ip, error = %error, "Trunk reload failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "message": error.to_string(),
                })),
            )
                .into_response()
        }
    }
}

async fn reload_routes_handler(State(state): State<AppState>, client_ip: ClientAddr) -> Response {
    info!(%client_ip, "Reload routing rules via /reload/routes endpoint");

    let config_override = match load_proxy_config_override(&state) {
        Ok(cfg) => cfg,
        Err(response) => return response,
    };

    match state
        .sip_server()
        .inner
        .data_context
        .reload_routes(true, config_override)
        .await
    {
        Ok(metrics) => {
            let total = metrics.total;
            Json(serde_json::json!({
                "status": "ok",
                "routes_reloaded": total,
                "metrics": metrics,
            }))
        }
        .into_response(),
        Err(error) => {
            warn!(%client_ip, error = %error, "Route reload failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "message": error.to_string(),
                })),
            )
                .into_response()
        }
    }
}

async fn reload_acl_handler(State(state): State<AppState>, client_ip: ClientAddr) -> Response {
    info!(%client_ip, "Reload ACL rules via /reload/acl endpoint");
    let context = state.sip_server().inner.data_context.clone();

    let config_override = match load_proxy_config_override(&state) {
        Ok(cfg) => cfg,
        Err(response) => return response,
    };

    match context.reload_acl_rules(true, config_override) {
        Ok(metrics) => {
            let total = metrics.total;
            let active_rules = context.acl_rules_snapshot();
            Json(serde_json::json!({
                "status": "ok",
                "acl_rules_reloaded": total,
                "metrics": metrics,
                "active_rules": active_rules,
            }))
        }
        .into_response(),
        Err(error) => {
            warn!(%client_ip, error = %error, "ACL reload failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "message": error.to_string(),
                })),
            )
                .into_response()
        }
    }
}

fn load_proxy_config_override(state: &AppState) -> Result<Option<Arc<ProxyConfig>>, Response> {
    let Some(path) = state.config_path.as_ref() else {
        return Ok(None);
    };

    match Config::load(path) {
        Ok(cfg) => Ok(Some(Arc::new(cfg.proxy))),
        Err(err) => {
            warn!(path = %path, ?err, "configuration reload failed during parsing");
            Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({
                    "status": "invalid",
                    "message": format!("Failed to load configuration: {}", err),
                })),
            )
                .into_response())
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct ReloadAppParams {
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    check_only: bool,
    #[serde(default)]
    dry_run: bool,
}

async fn reload_app_handler(
    State(state): State<AppState>,
    client_ip: ClientAddr,
    Query(params): Query<ReloadAppParams>,
) -> Response {
    let requested_mode = params.mode.as_deref();
    let check_only = params.check_only
        || params.dry_run
        || matches!(requested_mode, Some(mode) if mode.eq_ignore_ascii_case("check") || mode.eq_ignore_ascii_case("validate"));

    info!(%client_ip, check_only, "Reload application via /reload/app endpoint");

    let Some(config_path) = state.config_path.clone() else {
        warn!(%client_ip, "Reload rejected: configuration path unknown");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "error",
                "message": "Application was started without a configuration file path; reload is unavailable.",
            })),
        )
            .into_response();
    };

    let proposed = match crate::config::Config::load(&config_path) {
        Ok(cfg) => cfg,
        Err(err) => {
            warn!(%client_ip, path = %config_path, error = %err, "Configuration reload failed during parsing");
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({
                    "status": "invalid",
                    "errors": [{
                        "field": "config",
                        "message": format!("Failed to load configuration: {}", err),
                    }],
                })),
            )
                .into_response();
        }
    };

    if let Err(preflight_error) = preflight::validate_reload(&state, &proposed).await {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({
                "status": "invalid",
                "errors": preflight_error.issues,
            })),
        )
            .into_response();
    }

    if check_only {
        return Json(serde_json::json!({
            "status": "ok",
            "mode": "check",
            "message": "Configuration validated. Services not restarted.",
        }))
        .into_response();
    }

    state.reload_requested.store(true, Ordering::Relaxed);
    let cancel_token = state.token().clone();
    crate::utils::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        cancel_token.cancel();
    });

    Json(serde_json::json!({
        "status": "ok",
        "message": "Configuration validated. Restarting services with updated configuration.",
    }))
    .into_response()
}

#[derive(Deserialize)]
struct FrequencyLimitQuery {
    policy_id: Option<String>,
    scope: Option<String>,
    scope_value: Option<String>,
    limit_type: Option<String>,
}

async fn list_frequency_limits(
    State(state): State<AppState>,
    Query(params): Query<FrequencyLimitQuery>,
) -> Response {
    let Some(limiter) = state.sip_server().inner.frequency_limiter.as_ref() else {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({
                "status": "unavailable",
                "reason": "frequency_limiter_not_configured",
            })),
        )
            .into_response();
    };

    match limiter
        .list_limits(
            params.policy_id,
            params.scope,
            params.scope_value,
            params.limit_type,
        )
        .await
    {
        Ok(limits) => Json(limits).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "message": err.to_string(),
            })),
        )
            .into_response(),
    }
}

async fn clear_frequency_limits(
    State(state): State<AppState>,
    Query(params): Query<FrequencyLimitQuery>,
) -> Response {
    let Some(limiter) = state.sip_server().inner.frequency_limiter.as_ref() else {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({
                "status": "unavailable",
                "reason": "frequency_limiter_not_configured",
            })),
        )
            .into_response();
    };

    match limiter
        .clear_limits(
            params.policy_id,
            params.scope,
            params.scope_value,
            params.limit_type,
        )
        .await
    {
        Ok(deleted_count) => Json(serde_json::json!({
            "status": "ok",
            "deleted_count": deleted_count,
        }))
        .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "message": err.to_string(),
            })),
        )
            .into_response(),
    }
}

async fn backup_status_handler(State(state): State<AppState>) -> Response {
    let Some(ref backup_svc) = state.backup_service else {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({
                "status": "unavailable",
                "reason": "backup_not_configured",
                "message": "Add [backup] section with enabled = true to your config to enable backups",
            })),
        )
            .into_response();
    };

    let status = backup_svc.get_status();
    Json(serde_json::json!({
        "status": "ok",
        "backup": status,
    }))
    .into_response()
}

async fn backup_trigger_handler(
    State(state): State<AppState>,
    client_ip: ClientAddr,
) -> Response {
    let Some(ref backup_svc) = state.backup_service else {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({
                "status": "unavailable",
                "reason": "backup_not_configured",
                "message": "Add [backup] section with enabled = true to your config to enable backups",
            })),
        )
            .into_response();
    };

    info!(%client_ip, "Manual backup triggered via /backup/trigger endpoint");

    match backup_svc.trigger_backup().await {
        Ok(path) => Json(serde_json::json!({
            "status": "ok",
            "message": "Backup completed successfully",
            "path": path,
        }))
        .into_response(),
        Err(err) => {
            warn!(%client_ip, error = %err, "Manual backup failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "message": err,
                })),
            )
                .into_response()
        }
    }
}

async fn backup_health_handler(State(state): State<AppState>) -> Response {
    let Some(ref backup_svc) = state.backup_service else {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({
                "status": "unavailable",
                "reason": "backup_not_configured",
                "message": "Add [backup] section with enabled = true to your config to enable backups",
            })),
        )
            .into_response();
    };

    let health = backup_svc.check_health().await;
    Json(serde_json::json!({
        "status": "ok",
        "health": health,
    }))
    .into_response()
}

async fn backup_verify_handler(State(state): State<AppState>) -> Response {
    let Some(ref backup_svc) = state.backup_service else {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({
                "status": "unavailable",
                "reason": "backup_not_configured",
                "message": "Add [backup] section with enabled = true to your config to enable backups",
            })),
        )
            .into_response();
    };

    match backup_svc.verify_latest_backup().await {
        Ok(result) => Json(serde_json::json!({
            "status": "ok",
            "verification": result,
        }))
        .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "message": err,
            })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct BackupHistoryQuery {
    #[serde(default = "default_backup_history_limit")]
    limit: usize,
}

fn default_backup_history_limit() -> usize {
    20
}

async fn backup_history_handler(
    State(state): State<AppState>,
    Query(params): Query<BackupHistoryQuery>,
) -> Response {
    let Some(ref backup_svc) = state.backup_service else {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({
                "status": "unavailable",
                "reason": "backup_not_configured",
                "message": "Add [backup] section with enabled = true to your config to enable backups",
            })),
        )
            .into_response();
    };

    match backup_svc.get_history(params.limit).await {
        Ok(history) => Json(serde_json::json!({
            "status": "ok",
            "count": history.len(),
            "history": history,
        }))
        .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "message": err,
            })),
        )
            .into_response(),
    }
}

// --- Call monitoring endpoints ---

#[derive(Debug, Deserialize)]
struct MonitorModePayload {
    mode: MonitorMode,
    /// Optional identifier for the agent being monitored (for audit trail).
    #[serde(default)]
    agent_extension: Option<String>,
}

/// Convert a `MonitorMode` to its string representation for the audit log.
fn monitor_mode_name(mode: MonitorMode) -> &'static str {
    match mode {
        MonitorMode::SilentListen => "silent_listen",
        MonitorMode::Whisper => "whisper",
        MonitorMode::Barge => "barge",
    }
}

/// Check whether the request is authorized for call monitoring operations.
///
/// Monitoring requires *supervisor-level* access:
/// - When the console feature is enabled and a session cookie is present,
///   the user must be a superuser or staff member (i.e., have a supervisor
///   or manager role).
/// - When access is granted purely through the AMI IP allow-list (no session
///   cookie), the caller is treated as an administrator and is permitted.
///
/// Returns `Ok(user_identifier)` on success or an error `Response` on failure.
#[allow(unused_variables)]
async fn check_monitor_permission(
    state: &AppState,
    headers: &axum::http::HeaderMap,
    client_ip: &ClientAddr,
) -> Result<String, Response> {
    // When the console feature is compiled in, try to resolve the authenticated user.
    #[cfg(feature = "console")]
    {
        if let Some(console_state) = &state.console {
            if let Some(cookie_value) =
                crate::console::middleware::extract_session_cookie(headers)
            {
                match console_state.current_user(Some(&cookie_value)).await {
                    Ok(Some(user)) => {
                        // Monitoring requires supervisor/manager privileges:
                        // is_superuser or is_staff.
                        if user.is_superuser || user.is_staff {
                            return Ok(format!("{}(id:{})", user.username, user.id));
                        }
                        warn!(
                            %client_ip,
                            username = %user.username,
                            "Monitoring access denied: user lacks supervisor/manager role"
                        );
                        return Err((
                            StatusCode::FORBIDDEN,
                            Json(serde_json::json!({
                                "error": "Forbidden",
                                "message": "Supervisor or manager role required to use call monitoring",
                            })),
                        )
                            .into_response());
                    }
                    Ok(None) => {
                        // Cookie present but no valid/active user.
                        warn!(%client_ip, "Monitoring access denied: invalid or expired session");
                        return Err((
                            StatusCode::FORBIDDEN,
                            Json(serde_json::json!({
                                "error": "Forbidden",
                                "message": "Valid supervisor or manager session required to use call monitoring",
                            })),
                        )
                            .into_response());
                    }
                    Err(err) => {
                        warn!(%client_ip, error = %err, "Monitoring access denied: failed to resolve user");
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({
                                "error": "Internal error",
                                "message": "Failed to verify user session",
                            })),
                        )
                            .into_response());
                    }
                }
            }
        }
    }

    // No console session - access was granted via AMI IP allow-list, which
    // implies administrative / supervisor access.
    Ok(format!("ami_ip:{}", client_ip.ip()))
}

/// Insert a monitoring audit event into the database.
///
/// This is fire-and-forget: failures are logged but do not block the API response.
async fn record_monitoring_event(
    state: &AppState,
    session_id: &str,
    monitor_user_id: &str,
    agent_extension: &str,
    event_type: &str,
    monitor_mode: &str,
    details: Option<String>,
) {
    use crate::models::monitoring_event::ActiveModel;

    let now = Utc::now();
    let model = ActiveModel {
        id: Default::default(),
        session_id: Set(session_id.to_string()),
        monitor_user_id: Set(monitor_user_id.to_string()),
        agent_extension: Set(agent_extension.to_string()),
        event_type: Set(event_type.to_string()),
        monitor_mode: Set(monitor_mode.to_string()),
        timestamp: Set(now),
        details: Set(details),
    };

    if let Err(err) = model.insert(state.db()).await {
        warn!(
            session_id = %session_id,
            event_type = %event_type,
            error = %err,
            "Failed to persist monitoring audit event"
        );
    }
}

/// If notify_agent_on_monitor is enabled, log that monitoring is active.
fn maybe_log_agent_notification(state: &AppState, session_id: &str, mode: MonitorMode) {
    let notify = state
        .config()
        .monitoring
        .as_ref()
        .map(|m| m.notify_agent_on_monitor)
        .unwrap_or(false);

    if notify {
        info!(
            session_id = %session_id,
            mode = %monitor_mode_name(mode),
            "Agent notification: call monitoring is active (SIP notification is future work)"
        );
    }
}

async fn monitor_start_handler(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
    client_ip: ClientAddr,
    request: axum::extract::Request,
) -> Response {
    // Extract headers before consuming the request body.
    let headers = request.headers().clone();

    // Permission check
    let monitor_user = match check_monitor_permission(&state, &headers, &client_ip).await {
        Ok(user) => user,
        Err(resp) => return resp,
    };

    // Parse the JSON body
    let payload: MonitorModePayload = match axum::Json::from_request(request, &state).await {
        Ok(axum::Json(p)) => p,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Invalid request body: {}", err),
                })),
            )
                .into_response();
        }
    };

    let agent_ext = payload.agent_extension.clone().unwrap_or_default();

    let registry = state.sip_server().inner.active_call_registry.clone();
    let Some(handle) = registry.get_handle(&session_id) else {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "status": "error", "message": format!("Call session '{}' not found", session_id),
        }))).into_response();
    };
    match handle.send_command(SessionAction::MonitorStart { mode: payload.mode }) {
        Ok(_) => {
            let mode_str = monitor_mode_name(payload.mode);
            info!(
                session_id = %session_id,
                mode = %mode_str,
                monitor_user = %monitor_user,
                "Monitor start dispatched"
            );

            // Audit log
            record_monitoring_event(
                &state,
                &session_id,
                &monitor_user,
                &agent_ext,
                "monitor_start",
                mode_str,
                None,
            )
            .await;

            // Agent notification
            maybe_log_agent_notification(&state, &session_id, payload.mode);

            Json(serde_json::json!({
                "status": "ok",
                "session_id": session_id,
                "mode": payload.mode,
                "monitor_user": monitor_user,
            })).into_response()
        }
        Err(err) => (StatusCode::CONFLICT, Json(serde_json::json!({
            "status": "error", "message": format!("Failed: {}", err),
        }))).into_response(),
    }
}

async fn monitor_stop_handler(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
    client_ip: ClientAddr,
    request: axum::extract::Request,
) -> Response {
    // Extract headers before consuming the request.
    let headers = request.headers().clone();

    // Permission check
    let monitor_user = match check_monitor_permission(&state, &headers, &client_ip).await {
        Ok(user) => user,
        Err(resp) => return resp,
    };

    let registry = state.sip_server().inner.active_call_registry.clone();
    let Some(handle) = registry.get_handle(&session_id) else {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "status": "error", "message": format!("Call session '{}' not found", session_id),
        }))).into_response();
    };
    match handle.send_command(SessionAction::MonitorStop) {
        Ok(_) => {
            info!(
                session_id = %session_id,
                monitor_user = %monitor_user,
                "Monitor stop dispatched"
            );

            // Audit log
            record_monitoring_event(
                &state,
                &session_id,
                &monitor_user,
                "",
                "monitor_stop",
                "",
                None,
            )
            .await;

            Json(serde_json::json!({
                "status": "ok",
                "session_id": session_id,
                "monitor_user": monitor_user,
            })).into_response()
        }
        Err(err) => (StatusCode::CONFLICT, Json(serde_json::json!({
            "status": "error", "message": format!("Failed: {}", err),
        }))).into_response(),
    }
}

async fn monitor_set_mode_handler(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
    client_ip: ClientAddr,
    request: axum::extract::Request,
) -> Response {
    // Extract headers before consuming the request body.
    let headers = request.headers().clone();

    // Permission check
    let monitor_user = match check_monitor_permission(&state, &headers, &client_ip).await {
        Ok(user) => user,
        Err(resp) => return resp,
    };

    // Parse the JSON body
    let payload: MonitorModePayload = match axum::Json::from_request(request, &state).await {
        Ok(axum::Json(p)) => p,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Invalid request body: {}", err),
                })),
            )
                .into_response();
        }
    };

    let agent_ext = payload.agent_extension.clone().unwrap_or_default();

    let registry = state.sip_server().inner.active_call_registry.clone();
    let Some(handle) = registry.get_handle(&session_id) else {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "status": "error", "message": format!("Call session '{}' not found", session_id),
        }))).into_response();
    };
    match handle.send_command(SessionAction::MonitorSetMode { mode: payload.mode }) {
        Ok(_) => {
            let mode_str = monitor_mode_name(payload.mode);
            info!(
                session_id = %session_id,
                mode = %mode_str,
                monitor_user = %monitor_user,
                "Monitor mode change dispatched"
            );

            // Audit log
            record_monitoring_event(
                &state,
                &session_id,
                &monitor_user,
                &agent_ext,
                "mode_change",
                mode_str,
                None,
            )
            .await;

            Json(serde_json::json!({
                "status": "ok",
                "session_id": session_id,
                "mode": payload.mode,
                "monitor_user": monitor_user,
            })).into_response()
        }
        Err(err) => (StatusCode::CONFLICT, Json(serde_json::json!({
            "status": "error", "message": format!("Failed: {}", err),
        }))).into_response(),
    }
}
