//! CRUD handlers for `iiz.triggers` and its `trigger_conditions` / `trigger_actions` sub-resources.
//!
//! The top-level triggers CRUD uses the `crud_handlers!` macro.
//! The condition and action sub-resources require manual handlers because:
//! - They are scoped under a parent trigger via `/triggers/{trigger_id}/conditions` and `/actions`
//! - Path extractors need a parent_id + child_id tuple for single-resource routes
//!
//! All handlers use RLS-scoped connections for account_id filtering.

// --- Standard CRUD for triggers via macro ---
//
// The macro expands module-level `use` statements for axum extractors,
// diesel prelude, uuid, and all iiz API plumbing. The manual child handlers
// below reuse those imports.

use crate::iiz::models::automations::{NewTrigger, Trigger, UpdateTrigger};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::triggers,
    entity: Trigger,
    new_entity: NewTrigger,
    update_entity: UpdateTrigger,
);

// --- Manual handlers for the trigger_conditions sub-resource ---

use crate::iiz::models::automations::{NewTriggerCondition, TriggerCondition, UpdateTriggerCondition};

/// List conditions belonging to a specific trigger.
///
/// GET `/flows/triggers/{trigger_id}/conditions?page=1&per_page=25`
pub async fn list_conditions(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<TriggerCondition>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::trigger_conditions::dsl::*;

    let total: i64 = trigger_conditions
        .filter(trigger_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<TriggerCondition> = trigger_conditions
        .filter(trigger_id.eq(parent_id))
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

/// Get a single condition by ID within a trigger.
///
/// GET `/flows/triggers/{trigger_id}/conditions/{id}`
pub async fn get_condition(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<TriggerCondition>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::trigger_conditions::dsl::*;

    let item: TriggerCondition = trigger_conditions
        .filter(trigger_id.eq(parent_id))
        .filter(id.eq(child_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new condition in a trigger.
///
/// POST `/flows/triggers/{trigger_id}/conditions`
pub async fn create_condition(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewTriggerCondition>,
) -> Result<(axum::http::StatusCode, axum::Json<TriggerCondition>), ApiError> {
    payload.trigger_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: TriggerCondition =
        diesel::insert_into(crate::iiz::schema::iiz::trigger_conditions::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update a condition within a trigger.
///
/// PUT `/flows/triggers/{trigger_id}/conditions/{id}`
pub async fn update_condition(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateTriggerCondition>,
) -> Result<axum::Json<TriggerCondition>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::trigger_conditions::dsl::*;

    let item: TriggerCondition = diesel::update(
        trigger_conditions
            .filter(trigger_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete a condition from a trigger.
///
/// DELETE `/flows/triggers/{trigger_id}/conditions/{id}`
pub async fn delete_condition(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::trigger_conditions::dsl::*;

    diesel::update(
        trigger_conditions
            .filter(trigger_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// --- Manual handlers for the trigger_actions sub-resource ---

use crate::iiz::models::automations::{NewTriggerAction, TriggerAction, UpdateTriggerAction};

/// List actions belonging to a specific trigger.
///
/// GET `/flows/triggers/{trigger_id}/actions?page=1&per_page=25`
pub async fn list_actions(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<TriggerAction>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::trigger_actions::dsl::*;

    let total: i64 = trigger_actions
        .filter(trigger_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<TriggerAction> = trigger_actions
        .filter(trigger_id.eq(parent_id))
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

/// Get a single action by ID within a trigger.
///
/// GET `/flows/triggers/{trigger_id}/actions/{id}`
pub async fn get_action(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<TriggerAction>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::trigger_actions::dsl::*;

    let item: TriggerAction = trigger_actions
        .filter(trigger_id.eq(parent_id))
        .filter(id.eq(child_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new action in a trigger.
///
/// POST `/flows/triggers/{trigger_id}/actions`
pub async fn create_action(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewTriggerAction>,
) -> Result<(axum::http::StatusCode, axum::Json<TriggerAction>), ApiError> {
    payload.trigger_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: TriggerAction =
        diesel::insert_into(crate::iiz::schema::iiz::trigger_actions::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update an action within a trigger.
///
/// PUT `/flows/triggers/{trigger_id}/actions/{id}`
pub async fn update_action(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateTriggerAction>,
) -> Result<axum::Json<TriggerAction>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::trigger_actions::dsl::*;

    let item: TriggerAction = diesel::update(
        trigger_actions
            .filter(trigger_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete an action from a trigger.
///
/// DELETE `/flows/triggers/{trigger_id}/actions/{id}`
pub async fn delete_action(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::trigger_actions::dsl::*;

    diesel::update(
        trigger_actions
            .filter(trigger_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
