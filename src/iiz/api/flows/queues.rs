//! CRUD handlers for `iiz.queues` and its `queue_agents` sub-resource.
//!
//! The top-level queues CRUD uses the `crud_handlers!` macro.
//! The agent sub-resource requires manual handlers because:
//! - Agents are scoped under a parent queue via `/queues/{queue_id}/agents`
//! - Path extractors need a parent_id + agent_id tuple for single-resource routes
//!
//! All handlers use RLS-scoped connections for account_id filtering.

// --- Standard CRUD for queues via macro ---
//
// The macro expands module-level `use` statements for axum extractors,
// diesel prelude, uuid, and all iiz API plumbing. The manual agent handlers
// below reuse those imports.

use crate::iiz::models::flows::{NewQueue, Queue, UpdateQueue};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::queues,
    entity: Queue,
    new_entity: NewQueue,
    update_entity: UpdateQueue,
);

// --- Manual handlers for the agent sub-resource ---

use crate::iiz::models::flows::{NewQueueAgent, QueueAgent, UpdateQueueAgent};

/// List agents belonging to a specific queue.
///
/// GET `/flows/queues/{queue_id}/agents?page=1&per_page=25`
pub async fn list_agents(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<QueueAgent>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::queue_agents::dsl::*;

    let total: i64 = queue_agents
        .filter(queue_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<QueueAgent> = queue_agents
        .filter(queue_id.eq(parent_id))
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

/// Get a single agent by ID within a queue.
///
/// GET `/flows/queues/{queue_id}/agents/{id}`
pub async fn get_agent(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<QueueAgent>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::queue_agents::dsl::*;

    let item: QueueAgent = queue_agents
        .filter(queue_id.eq(parent_id))
        .filter(id.eq(child_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new agent assignment in a queue.
///
/// POST `/flows/queues/{queue_id}/agents`
///
/// The `queue_id` from the URL path is injected into the payload to ensure
/// consistency -- the caller does not need to include it in the JSON body.
pub async fn create_agent(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewQueueAgent>,
) -> Result<(axum::http::StatusCode, axum::Json<QueueAgent>), ApiError> {
    // Override queue_id from URL path for consistency
    payload.queue_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: QueueAgent =
        diesel::insert_into(crate::iiz::schema::iiz::queue_agents::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update an agent assignment within a queue.
///
/// PUT `/flows/queues/{queue_id}/agents/{id}`
pub async fn update_agent(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateQueueAgent>,
) -> Result<axum::Json<QueueAgent>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::queue_agents::dsl::*;

    let item: QueueAgent = diesel::update(
        queue_agents
            .filter(queue_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete an agent assignment from a queue.
///
/// DELETE `/flows/queues/{queue_id}/agents/{id}`
pub async fn delete_agent(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::queue_agents::dsl::*;

    diesel::update(
        queue_agents
            .filter(queue_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
