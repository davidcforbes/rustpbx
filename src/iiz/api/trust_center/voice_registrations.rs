//! CRUD handlers for `iiz.voice_registrations` and its `voice_registration_history` sub-resource.
//!
//! The top-level voice_registrations CRUD uses the `crud_handlers!` macro.
//! The history sub-resource requires manual handlers because:
//! - History entries are scoped under a parent registration via `/voice-registrations/{reg_id}/history`
//! - History is an event log: only list and create are supported (no update/delete)
//!
//! All handlers use RLS-scoped connections for account_id filtering.

// --- Standard CRUD for voice_registrations via macro ---
//
// The macro expands module-level `use` statements for axum extractors,
// diesel prelude, uuid, and all iiz API plumbing. The manual history handlers
// below reuse those imports.

use crate::iiz::models::trust_center::{NewVoiceRegistration, UpdateVoiceRegistration, VoiceRegistration};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::voice_registrations,
    entity: VoiceRegistration,
    new_entity: NewVoiceRegistration,
    update_entity: UpdateVoiceRegistration,
);

// --- Manual handlers for the history sub-resource ---

use crate::iiz::models::trust_center::{NewVoiceRegistrationHistoryEntry, VoiceRegistrationHistoryEntry};

/// List history entries for a specific voice registration.
///
/// GET `/trust-center/voice-registrations/{reg_id}/history?page=1&per_page=25`
pub async fn list_history(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<VoiceRegistrationHistoryEntry>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::voice_registration_history::dsl::*;

    let total: i64 = voice_registration_history
        .filter(registration_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<VoiceRegistrationHistoryEntry> = voice_registration_history
        .filter(registration_id.eq(parent_id))
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

/// Create a new history entry for a voice registration.
///
/// POST `/trust-center/voice-registrations/{reg_id}/history`
///
/// The `registration_id` from the URL path is injected into the payload to ensure
/// consistency -- the caller does not need to include it in the JSON body.
pub async fn create_history(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewVoiceRegistrationHistoryEntry>,
) -> Result<(axum::http::StatusCode, axum::Json<VoiceRegistrationHistoryEntry>), ApiError> {
    // Override registration_id from URL path for consistency
    payload.registration_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: VoiceRegistrationHistoryEntry =
        diesel::insert_into(crate::iiz::schema::iiz::voice_registration_history::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}
