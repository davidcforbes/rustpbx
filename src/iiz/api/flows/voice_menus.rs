//! CRUD handlers for `iiz.voice_menus` and its `voice_menu_options` sub-resource.
//!
//! The top-level voice_menus CRUD uses the `crud_handlers!` macro.
//! The option sub-resource requires manual handlers because:
//! - Options are scoped under a parent menu via `/voice-menus/{menu_id}/options`
//! - Path extractors need a parent_id + option_id tuple for single-resource routes
//!
//! All handlers use RLS-scoped connections for account_id filtering.

// --- Standard CRUD for voice_menus via macro ---
//
// The macro expands module-level `use` statements for axum extractors,
// diesel prelude, uuid, and all iiz API plumbing. The manual option handlers
// below reuse those imports.

use crate::iiz::models::flows::{NewVoiceMenu, UpdateVoiceMenu, VoiceMenu};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::voice_menus,
    entity: VoiceMenu,
    new_entity: NewVoiceMenu,
    update_entity: UpdateVoiceMenu,
);

// --- Manual handlers for the option sub-resource ---

use crate::iiz::models::flows::{NewVoiceMenuOption, UpdateVoiceMenuOption, VoiceMenuOption};

/// List options belonging to a specific voice menu.
///
/// GET `/flows/voice-menus/{menu_id}/options?page=1&per_page=25`
pub async fn list_options(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<VoiceMenuOption>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::voice_menu_options::dsl::*;

    let total: i64 = voice_menu_options
        .filter(menu_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<VoiceMenuOption> = voice_menu_options
        .filter(menu_id.eq(parent_id))
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

/// Get a single option by ID within a voice menu.
///
/// GET `/flows/voice-menus/{menu_id}/options/{id}`
pub async fn get_option(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, option_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<VoiceMenuOption>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::voice_menu_options::dsl::*;

    let item: VoiceMenuOption = voice_menu_options
        .filter(menu_id.eq(parent_id))
        .filter(id.eq(option_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new option in a voice menu.
///
/// POST `/flows/voice-menus/{menu_id}/options`
///
/// The `menu_id` from the URL path is injected into the payload to ensure
/// consistency -- the caller does not need to include it in the JSON body.
pub async fn create_option(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewVoiceMenuOption>,
) -> Result<(axum::http::StatusCode, axum::Json<VoiceMenuOption>), ApiError> {
    // Override menu_id from URL path for consistency
    payload.menu_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: VoiceMenuOption =
        diesel::insert_into(crate::iiz::schema::iiz::voice_menu_options::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update an option within a voice menu.
///
/// PUT `/flows/voice-menus/{menu_id}/options/{id}`
pub async fn update_option(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, option_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateVoiceMenuOption>,
) -> Result<axum::Json<VoiceMenuOption>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::voice_menu_options::dsl::*;

    let item: VoiceMenuOption = diesel::update(
        voice_menu_options
            .filter(menu_id.eq(parent_id))
            .filter(id.eq(option_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete an option from a voice menu.
///
/// DELETE `/flows/voice-menus/{menu_id}/options/{id}`
pub async fn delete_option(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, option_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::voice_menu_options::dsl::*;

    diesel::update(
        voice_menu_options
            .filter(menu_id.eq(parent_id))
            .filter(id.eq(option_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
