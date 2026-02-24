use crate::config::VoicemailConfig;
use crate::console::{ConsoleState, middleware::AuthRequired};
use crate::models::{
    voicemail::{
        ActiveModel as VoicemailActiveModel, Column as VoicemailColumn,
        Entity as VoicemailEntity,
    },
    voicemail_greeting::{
        ActiveModel as GreetingActiveModel, Column as GreetingColumn,
        Entity as GreetingEntity,
    },
};
use axum::{
    Json, Router,
    extract::{Multipart, Path as AxumPath, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use sea_orm::sea_query::Order;
use serde_json::json;
use std::sync::Arc;
use tracing::warn;

fn voicemail_config(state: &ConsoleState) -> VoicemailConfig {
    state
        .app_state()
        .and_then(|app| app.config().voicemail.clone())
        .unwrap_or_default()
}

/// Derive mailbox_id from the logged-in user's username.
fn mailbox_for_user(user: &crate::models::user::Model) -> String {
    user.username.clone()
}

pub fn urls() -> Router<Arc<ConsoleState>> {
    Router::new()
        .route("/voicemail", get(page_voicemail))
        .route("/voicemail/greetings", get(list_greetings))
        .route("/voicemail/greetings/upload", post(upload_greeting))
        .route(
            "/voicemail/greetings/{id}/activate",
            post(activate_greeting),
        )
        .route("/voicemail/greetings/{id}", delete(delete_greeting))
        .route("/voicemail/messages", get(list_messages))
        .route("/voicemail/messages/{id}/read", post(mark_message_read))
        .route("/voicemail/messages/{id}", delete(delete_message))
}

// ---------------------------------------------------------------------------
// Page handler
// ---------------------------------------------------------------------------

async fn page_voicemail(
    State(state): State<Arc<ConsoleState>>,
    AuthRequired(user): AuthRequired,
) -> Response {
    let mailbox_id = mailbox_for_user(&user);
    let vm_config = voicemail_config(&state);

    // Count unread messages
    let unread_count = VoicemailEntity::find()
        .filter(VoicemailColumn::MailboxId.eq(&mailbox_id))
        .filter(VoicemailColumn::IsRead.eq(false))
        .filter(VoicemailColumn::DeletedAt.is_null())
        .count(state.db())
        .await
        .unwrap_or(0);

    state.render(
        "console/voicemail.html",
        json!({
            "nav_active": "voicemail",
            "username": user.username,
            "email": user.email,
            "mailbox_id": mailbox_id,
            "unread_count": unread_count,
            "vm_enabled": vm_config.enabled,
        }),
    )
}

// ---------------------------------------------------------------------------
// Greeting endpoints
// ---------------------------------------------------------------------------

async fn list_greetings(
    State(state): State<Arc<ConsoleState>>,
    AuthRequired(user): AuthRequired,
) -> Response {
    let mailbox_id = mailbox_for_user(&user);

    let greetings = match GreetingEntity::find()
        .filter(GreetingColumn::MailboxId.eq(&mailbox_id))
        .order_by(GreetingColumn::CreatedAt, Order::Desc)
        .all(state.db())
        .await
    {
        Ok(items) => items,
        Err(err) => {
            warn!("failed to list greetings for {}: {}", mailbox_id, err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": err.to_string()})),
            )
                .into_response();
        }
    };

    Json(json!({ "items": greetings })).into_response()
}

async fn upload_greeting(
    State(state): State<Arc<ConsoleState>>,
    AuthRequired(user): AuthRequired,
    mut multipart: Multipart,
) -> Response {
    let mailbox_id = mailbox_for_user(&user);
    let vm_config = voicemail_config(&state);

    let mut greeting_type = "standard".to_string();
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "greeting_type" => {
                if let Ok(text) = field.text().await {
                    let trimmed = text.trim().to_string();
                    if !trimmed.is_empty() {
                        greeting_type = trimmed;
                    }
                }
            }
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                match field.bytes().await {
                    Ok(bytes) => {
                        file_data = Some(bytes.to_vec());
                    }
                    Err(err) => {
                        warn!("failed to read uploaded greeting file: {}", err);
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(json!({"message": "Failed to read uploaded file"})),
                        )
                            .into_response();
                    }
                }
            }
            _ => {
                // skip unknown fields
            }
        }
    }

    let data = match file_data {
        Some(d) if !d.is_empty() => d,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"message": "No audio file provided"})),
            )
                .into_response();
        }
    };

    // Validate file extension
    let ext = file_name
        .as_deref()
        .and_then(|n| n.rsplit('.').next())
        .unwrap_or("wav")
        .to_lowercase();
    if !matches!(ext.as_str(), "wav" | "mp3" | "ogg" | "webm") {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"message": "Unsupported audio format. Use WAV, MP3, OGG, or WebM."})),
        )
            .into_response();
    }

    // Ensure greeting directory exists
    let greeting_dir = std::path::PathBuf::from(&vm_config.greeting_path).join(&mailbox_id);
    if let Err(err) = tokio::fs::create_dir_all(&greeting_dir).await {
        warn!("failed to create greeting directory {:?}: {}", greeting_dir, err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Failed to create greeting storage directory"})),
        )
            .into_response();
    }

    // Generate unique filename
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let dest_filename = format!("{}_{}.{}", greeting_type, timestamp, ext);
    let dest_path = greeting_dir.join(&dest_filename);

    if let Err(err) = tokio::fs::write(&dest_path, &data).await {
        warn!("failed to write greeting file {:?}: {}", dest_path, err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Failed to save greeting file"})),
        )
            .into_response();
    }

    let recording_path = dest_path.to_string_lossy().to_string();
    let now = Utc::now();

    let active = GreetingActiveModel {
        mailbox_id: Set(mailbox_id.clone()),
        greeting_type: Set(greeting_type),
        recording_path: Set(recording_path),
        is_active: Set(false),
        created_at: Set(now),
        ..Default::default()
    };

    match active.insert(state.db()).await {
        Ok(model) => {
            Json(json!({"status": "ok", "id": model.id})).into_response()
        }
        Err(err) => {
            warn!("failed to insert greeting record: {}", err);
            // Clean up the file we just wrote
            let _ = tokio::fs::remove_file(&dest_path).await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": err.to_string()})),
            )
                .into_response()
        }
    }
}

async fn activate_greeting(
    AxumPath(id): AxumPath<i64>,
    State(state): State<Arc<ConsoleState>>,
    AuthRequired(user): AuthRequired,
) -> Response {
    let mailbox_id = mailbox_for_user(&user);
    let db = state.db();

    // Fetch the greeting to activate
    let target = match GreetingEntity::find_by_id(id).one(db).await {
        Ok(Some(g)) => g,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"message": "Greeting not found"})),
            )
                .into_response();
        }
        Err(err) => {
            warn!("failed to load greeting {}: {}", id, err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": err.to_string()})),
            )
                .into_response();
        }
    };

    if target.mailbox_id != mailbox_id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"message": "Greeting does not belong to your mailbox"})),
        )
            .into_response();
    }

    // Deactivate all greetings of the same type for this mailbox
    let all_same_type = match GreetingEntity::find()
        .filter(GreetingColumn::MailboxId.eq(&mailbox_id))
        .filter(GreetingColumn::GreetingType.eq(&target.greeting_type))
        .filter(GreetingColumn::IsActive.eq(true))
        .all(db)
        .await
    {
        Ok(items) => items,
        Err(err) => {
            warn!("failed to query greetings for deactivation: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": err.to_string()})),
            )
                .into_response();
        }
    };

    for g in all_same_type {
        let mut active: GreetingActiveModel = g.into();
        active.is_active = Set(false);
        if let Err(err) = active.update(db).await {
            warn!("failed to deactivate greeting: {}", err);
        }
    }

    // Activate the target
    let mut active: GreetingActiveModel = target.into();
    active.is_active = Set(true);
    if let Err(err) = active.update(db).await {
        warn!("failed to activate greeting {}: {}", id, err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": err.to_string()})),
        )
            .into_response();
    }

    Json(json!({"status": "ok"})).into_response()
}

async fn delete_greeting(
    AxumPath(id): AxumPath<i64>,
    State(state): State<Arc<ConsoleState>>,
    AuthRequired(user): AuthRequired,
) -> Response {
    let mailbox_id = mailbox_for_user(&user);
    let db = state.db();

    let greeting = match GreetingEntity::find_by_id(id).one(db).await {
        Ok(Some(g)) => g,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"message": "Greeting not found"})),
            )
                .into_response();
        }
        Err(err) => {
            warn!("failed to load greeting {} for deletion: {}", id, err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": err.to_string()})),
            )
                .into_response();
        }
    };

    if greeting.mailbox_id != mailbox_id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"message": "Greeting does not belong to your mailbox"})),
        )
            .into_response();
    }

    // Delete the file on disk
    let path = std::path::Path::new(&greeting.recording_path);
    if path.exists() {
        if let Err(err) = tokio::fs::remove_file(path).await {
            warn!("failed to remove greeting file {:?}: {}", path, err);
        }
    }

    // Delete the database record
    match GreetingEntity::delete_by_id(id).exec(db).await {
        Ok(r) => Json(json!({"status": "ok", "rows_affected": r.rows_affected})).into_response(),
        Err(err) => {
            warn!("failed to delete greeting {}: {}", id, err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": err.to_string()})),
            )
                .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Message endpoints
// ---------------------------------------------------------------------------

async fn list_messages(
    State(state): State<Arc<ConsoleState>>,
    AuthRequired(user): AuthRequired,
) -> Response {
    let mailbox_id = mailbox_for_user(&user);

    let messages = match VoicemailEntity::find()
        .filter(VoicemailColumn::MailboxId.eq(&mailbox_id))
        .filter(VoicemailColumn::DeletedAt.is_null())
        .order_by(VoicemailColumn::CreatedAt, Order::Desc)
        .all(state.db())
        .await
    {
        Ok(items) => items,
        Err(err) => {
            warn!("failed to list voicemails for {}: {}", mailbox_id, err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": err.to_string()})),
            )
                .into_response();
        }
    };

    let unread_count = messages.iter().filter(|m| !m.is_read).count();

    Json(json!({
        "items": messages,
        "unread_count": unread_count,
    }))
    .into_response()
}

async fn mark_message_read(
    AxumPath(id): AxumPath<i64>,
    State(state): State<Arc<ConsoleState>>,
    AuthRequired(user): AuthRequired,
) -> Response {
    let mailbox_id = mailbox_for_user(&user);
    let db = state.db();

    let msg = match VoicemailEntity::find_by_id(id).one(db).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"message": "Message not found"})),
            )
                .into_response();
        }
        Err(err) => {
            warn!("failed to load voicemail {}: {}", id, err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": err.to_string()})),
            )
                .into_response();
        }
    };

    if msg.mailbox_id != mailbox_id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"message": "Message does not belong to your mailbox"})),
        )
            .into_response();
    }

    let mut active: VoicemailActiveModel = msg.into();
    active.is_read = Set(true);
    active.updated_at = Set(Utc::now());

    if let Err(err) = active.update(db).await {
        warn!("failed to mark voicemail {} as read: {}", id, err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": err.to_string()})),
        )
            .into_response();
    }

    Json(json!({"status": "ok"})).into_response()
}

async fn delete_message(
    AxumPath(id): AxumPath<i64>,
    State(state): State<Arc<ConsoleState>>,
    AuthRequired(user): AuthRequired,
) -> Response {
    let mailbox_id = mailbox_for_user(&user);
    let db = state.db();

    let msg = match VoicemailEntity::find_by_id(id).one(db).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"message": "Message not found"})),
            )
                .into_response();
        }
        Err(err) => {
            warn!("failed to load voicemail {} for deletion: {}", id, err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": err.to_string()})),
            )
                .into_response();
        }
    };

    if msg.mailbox_id != mailbox_id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"message": "Message does not belong to your mailbox"})),
        )
            .into_response();
    }

    // Soft-delete by setting deleted_at
    let mut active: VoicemailActiveModel = msg.into();
    active.deleted_at = Set(Some(Utc::now()));
    active.updated_at = Set(Utc::now());

    if let Err(err) = active.update(db).await {
        warn!("failed to soft-delete voicemail {}: {}", id, err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": err.to_string()})),
        )
            .into_response();
    }

    Json(json!({"status": "ok"})).into_response()
}
