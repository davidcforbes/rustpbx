//! CRUD handlers for `iiz.routing_tables` and its `routing_table_routes` sub-resource.
//!
//! The top-level routing_tables CRUD uses the `crud_handlers!` macro.
//! The route sub-resource requires manual handlers because:
//! - Routes are scoped under a parent table via `/routing-tables/{table_id}/routes`
//! - Path extractors need a parent_id + route_id tuple for single-resource routes
//!
//! All handlers use RLS-scoped connections for account_id filtering.

// --- Standard CRUD for routing_tables via macro ---
//
// The macro expands module-level `use` statements for axum extractors,
// diesel prelude, uuid, and all iiz API plumbing. The manual route handlers
// below reuse those imports.

use crate::iiz::models::flows::{NewRoutingTable, RoutingTable, UpdateRoutingTable};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::routing_tables,
    entity: RoutingTable,
    new_entity: NewRoutingTable,
    update_entity: UpdateRoutingTable,
);

// --- Manual handlers for the route sub-resource ---

use crate::iiz::models::flows::{NewRoutingTableRoute, RoutingTableRoute, UpdateRoutingTableRoute};

/// List routes belonging to a specific routing table.
///
/// GET `/flows/routing-tables/{table_id}/routes?page=1&per_page=25`
pub async fn list_routes(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<RoutingTableRoute>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::routing_table_routes::dsl::*;

    let total: i64 = routing_table_routes
        .filter(table_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<RoutingTableRoute> = routing_table_routes
        .filter(table_id.eq(parent_id))
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

/// Get a single route by ID within a routing table.
///
/// GET `/flows/routing-tables/{table_id}/routes/{id}`
pub async fn get_route(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, route_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<RoutingTableRoute>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::routing_table_routes::dsl::*;

    let item: RoutingTableRoute = routing_table_routes
        .filter(table_id.eq(parent_id))
        .filter(id.eq(route_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new route in a routing table.
///
/// POST `/flows/routing-tables/{table_id}/routes`
///
/// The `table_id` from the URL path is injected into the payload to ensure
/// consistency -- the caller does not need to include it in the JSON body.
pub async fn create_route(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewRoutingTableRoute>,
) -> Result<(axum::http::StatusCode, axum::Json<RoutingTableRoute>), ApiError> {
    // Override table_id from URL path for consistency
    payload.table_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: RoutingTableRoute =
        diesel::insert_into(crate::iiz::schema::iiz::routing_table_routes::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update a route within a routing table.
///
/// PUT `/flows/routing-tables/{table_id}/routes/{id}`
pub async fn update_route(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, route_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateRoutingTableRoute>,
) -> Result<axum::Json<RoutingTableRoute>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::routing_table_routes::dsl::*;

    let item: RoutingTableRoute = diesel::update(
        routing_table_routes
            .filter(table_id.eq(parent_id))
            .filter(id.eq(route_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete a route from a routing table.
///
/// DELETE `/flows/routing-tables/{table_id}/routes/{id}`
pub async fn delete_route(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, route_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::routing_table_routes::dsl::*;

    diesel::update(
        routing_table_routes
            .filter(table_id.eq(parent_id))
            .filter(id.eq(route_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
