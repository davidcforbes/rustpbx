//! CRUD handlers for `iiz.voicemail_boxes` and its `voicemail_messages` sub-resource.
//!
//! The top-level voicemail_boxes CRUD uses the `crud_handlers!` macro.
//! The message sub-resource requires manual handlers because:
//! - Messages are scoped under a parent mailbox via `/voicemails/{mailbox_id}/messages`
//! - Path extractors need a parent_id + message_id tuple for single-resource routes
//!
//! All handlers use RLS-scoped connections for account_id filtering.

// --- Standard CRUD for voicemail_boxes via macro ---
//
// The macro expands module-level `use` statements for axum extractors,
// diesel prelude, uuid, and all iiz API plumbing. The manual message handlers
// below reuse those imports.

use crate::iiz::models::flows::{NewVoicemailBox, UpdateVoicemailBox, VoicemailBox};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::voicemail_boxes,
    entity: VoicemailBox,
    new_entity: NewVoicemailBox,
    update_entity: UpdateVoicemailBox,
);

// --- Manual handlers for the message sub-resource ---

use crate::iiz::models::flows::{
    NewVoicemailMessage, UpdateVoicemailMessage, VoicemailMessage,
};

/// List messages belonging to a specific voicemail box.
///
/// GET `/flows/voicemails/{mailbox_id}/messages?page=1&per_page=25`
pub async fn list_messages(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<VoicemailMessage>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::voicemail_messages::dsl::*;

    let total: i64 = voicemail_messages
        .filter(mailbox_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<VoicemailMessage> = voicemail_messages
        .filter(mailbox_id.eq(parent_id))
        .order(created_at.desc())
        .offset(offset)
        .limit(limit)
        .load(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let meta = PaginationMeta::new(params.page.max(1), limit, total);
    Ok(axum::Json(ListResponse {
        pagination: meta,
        items,
    }))
}

/// Get a single message by ID within a voicemail box.
///
/// GET `/flows/voicemails/{mailbox_id}/messages/{id}`
pub async fn get_message(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<VoicemailMessage>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::voicemail_messages::dsl::*;

    let item: VoicemailMessage = voicemail_messages
        .filter(mailbox_id.eq(parent_id))
        .filter(id.eq(message_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new message in a voicemail box.
///
/// POST `/flows/voicemails/{mailbox_id}/messages`
///
/// The `mailbox_id` from the URL path is injected into the payload to ensure
/// consistency -- the caller does not need to include it in the JSON body.
pub async fn create_message(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewVoicemailMessage>,
) -> Result<(axum::http::StatusCode, axum::Json<VoicemailMessage>), ApiError> {
    // Override mailbox_id from URL path for consistency
    payload.mailbox_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: VoicemailMessage =
        diesel::insert_into(crate::iiz::schema::iiz::voicemail_messages::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update a message within a voicemail box.
///
/// PUT `/flows/voicemails/{mailbox_id}/messages/{id}`
///
/// Typically only `is_read` is toggled by the user.
pub async fn update_message(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, message_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateVoicemailMessage>,
) -> Result<axum::Json<VoicemailMessage>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::voicemail_messages::dsl::*;

    let item: VoicemailMessage = diesel::update(
        voicemail_messages
            .filter(mailbox_id.eq(parent_id))
            .filter(id.eq(message_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete a message from a voicemail box.
///
/// DELETE `/flows/voicemails/{mailbox_id}/messages/{id}`
pub async fn delete_message(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::voicemail_messages::dsl::*;

    diesel::update(
        voicemail_messages
            .filter(mailbox_id.eq(parent_id))
            .filter(id.eq(message_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
