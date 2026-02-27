//! CRUD handlers for `iiz.workflows` and its `workflow_nodes` / `workflow_edges` sub-resources.
//!
//! The top-level workflows CRUD uses the `crud_handlers!` macro.
//! The node and edge sub-resources require manual handlers because:
//! - They are scoped under a parent workflow via `/workflows/{workflow_id}/nodes` and `/edges`
//! - Path extractors need a parent_id + child_id tuple for single-resource routes
//!
//! All handlers use RLS-scoped connections for account_id filtering.

// --- Standard CRUD for workflows via macro ---
//
// The macro expands module-level `use` statements for axum extractors,
// diesel prelude, uuid, and all iiz API plumbing. The manual child handlers
// below reuse those imports.

use crate::iiz::models::automations::{NewWorkflow, UpdateWorkflow, Workflow};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::workflows,
    entity: Workflow,
    new_entity: NewWorkflow,
    update_entity: UpdateWorkflow,
);

// --- Manual handlers for the workflow_nodes sub-resource ---

use crate::iiz::models::automations::{NewWorkflowNode, UpdateWorkflowNode, WorkflowNode};

/// List nodes belonging to a specific workflow.
///
/// GET `/flows/workflows/{workflow_id}/nodes?page=1&per_page=25`
pub async fn list_nodes(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<WorkflowNode>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::workflow_nodes::dsl::*;

    let total: i64 = workflow_nodes
        .filter(workflow_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<WorkflowNode> = workflow_nodes
        .filter(workflow_id.eq(parent_id))
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

/// Get a single node by ID within a workflow.
///
/// GET `/flows/workflows/{workflow_id}/nodes/{id}`
pub async fn get_node(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<WorkflowNode>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::workflow_nodes::dsl::*;

    let item: WorkflowNode = workflow_nodes
        .filter(workflow_id.eq(parent_id))
        .filter(id.eq(child_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new node in a workflow.
///
/// POST `/flows/workflows/{workflow_id}/nodes`
pub async fn create_node(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewWorkflowNode>,
) -> Result<(axum::http::StatusCode, axum::Json<WorkflowNode>), ApiError> {
    payload.workflow_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: WorkflowNode =
        diesel::insert_into(crate::iiz::schema::iiz::workflow_nodes::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update a node within a workflow.
///
/// PUT `/flows/workflows/{workflow_id}/nodes/{id}`
pub async fn update_node(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateWorkflowNode>,
) -> Result<axum::Json<WorkflowNode>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::workflow_nodes::dsl::*;

    let item: WorkflowNode = diesel::update(
        workflow_nodes
            .filter(workflow_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete a node from a workflow.
///
/// DELETE `/flows/workflows/{workflow_id}/nodes/{id}`
pub async fn delete_node(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::workflow_nodes::dsl::*;

    diesel::update(
        workflow_nodes
            .filter(workflow_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// --- Manual handlers for the workflow_edges sub-resource ---

use crate::iiz::models::automations::{NewWorkflowEdge, UpdateWorkflowEdge, WorkflowEdge};

/// List edges belonging to a specific workflow.
///
/// GET `/flows/workflows/{workflow_id}/edges?page=1&per_page=25`
pub async fn list_edges(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<WorkflowEdge>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::workflow_edges::dsl::*;

    let total: i64 = workflow_edges
        .filter(workflow_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<WorkflowEdge> = workflow_edges
        .filter(workflow_id.eq(parent_id))
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

/// Get a single edge by ID within a workflow.
///
/// GET `/flows/workflows/{workflow_id}/edges/{id}`
pub async fn get_edge(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<WorkflowEdge>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::workflow_edges::dsl::*;

    let item: WorkflowEdge = workflow_edges
        .filter(workflow_id.eq(parent_id))
        .filter(id.eq(child_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new edge in a workflow.
///
/// POST `/flows/workflows/{workflow_id}/edges`
pub async fn create_edge(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewWorkflowEdge>,
) -> Result<(axum::http::StatusCode, axum::Json<WorkflowEdge>), ApiError> {
    payload.workflow_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: WorkflowEdge =
        diesel::insert_into(crate::iiz::schema::iiz::workflow_edges::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update an edge within a workflow.
///
/// PUT `/flows/workflows/{workflow_id}/edges/{id}`
pub async fn update_edge(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateWorkflowEdge>,
) -> Result<axum::Json<WorkflowEdge>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::workflow_edges::dsl::*;

    let item: WorkflowEdge = diesel::update(
        workflow_edges
            .filter(workflow_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete an edge from a workflow.
///
/// DELETE `/flows/workflows/{workflow_id}/edges/{id}`
pub async fn delete_edge(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::workflow_edges::dsl::*;

    diesel::update(
        workflow_edges
            .filter(workflow_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
