//! CRUD handlers for `iiz.lead_reactor_configs` and its `lead_reactor_actions` sub-resource.
//!
//! The top-level lead reactor config CRUD uses the `crud_handlers!` macro.
//! The action sub-resource requires manual handlers because:
//! - Actions are scoped under a parent config via `/lead-reactor/{config_id}/actions`
//! - Path extractors need a parent_id + child_id tuple for single-resource routes
//!
//! All handlers use RLS-scoped connections for account_id filtering.

// --- Standard CRUD for lead_reactor_configs via macro ---
//
// The macro expands module-level `use` statements for axum extractors,
// diesel prelude, uuid, and all iiz API plumbing. The manual child handlers
// below reuse those imports.

use crate::iiz::models::engagement::{LeadReactorConfig, NewLeadReactorConfig, UpdateLeadReactorConfig};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::lead_reactor_configs,
    entity: LeadReactorConfig,
    new_entity: NewLeadReactorConfig,
    update_entity: UpdateLeadReactorConfig,
);

// --- Manual handlers for the lead_reactor_actions sub-resource ---

use crate::iiz::models::engagement::{LeadReactorAction, NewLeadReactorAction, UpdateLeadReactorAction};

/// List actions belonging to a specific lead reactor config.
///
/// GET `/flows/lead-reactor/{config_id}/actions?page=1&per_page=25`
pub async fn list_actions(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<LeadReactorAction>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::lead_reactor_actions::dsl::*;

    let total: i64 = lead_reactor_actions
        .filter(config_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<LeadReactorAction> = lead_reactor_actions
        .filter(config_id.eq(parent_id))
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

/// Get a single action by ID within a lead reactor config.
///
/// GET `/flows/lead-reactor/{config_id}/actions/{id}`
pub async fn get_action(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<LeadReactorAction>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::lead_reactor_actions::dsl::*;

    let item: LeadReactorAction = lead_reactor_actions
        .filter(config_id.eq(parent_id))
        .filter(id.eq(child_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new action for a lead reactor config.
///
/// POST `/flows/lead-reactor/{config_id}/actions`
pub async fn create_action(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewLeadReactorAction>,
) -> Result<(axum::http::StatusCode, axum::Json<LeadReactorAction>), ApiError> {
    payload.config_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: LeadReactorAction =
        diesel::insert_into(crate::iiz::schema::iiz::lead_reactor_actions::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update an action within a lead reactor config.
///
/// PUT `/flows/lead-reactor/{config_id}/actions/{id}`
pub async fn update_action(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateLeadReactorAction>,
) -> Result<axum::Json<LeadReactorAction>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::lead_reactor_actions::dsl::*;

    let item: LeadReactorAction = diesel::update(
        lead_reactor_actions
            .filter(config_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete an action from a lead reactor config.
///
/// DELETE `/flows/lead-reactor/{config_id}/actions/{id}`
pub async fn delete_action(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::lead_reactor_actions::dsl::*;

    diesel::update(
        lead_reactor_actions
            .filter(config_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
