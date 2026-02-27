//! CRUD handlers for `iiz.geo_routers` and its `geo_router_rules` sub-resource.
//!
//! The top-level geo_routers CRUD uses the `crud_handlers!` macro.
//! The rule sub-resource requires manual handlers because:
//! - Rules are scoped under a parent router via `/geo-routers/{router_id}/rules`
//! - Path extractors need a parent_id + rule_id tuple for single-resource routes
//!
//! All handlers use RLS-scoped connections for account_id filtering.

// --- Standard CRUD for geo_routers via macro ---
//
// The macro expands module-level `use` statements for axum extractors,
// diesel prelude, uuid, and all iiz API plumbing. The manual rule handlers
// below reuse those imports.

use crate::iiz::models::flows::{GeoRouter, NewGeoRouter, UpdateGeoRouter};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::geo_routers,
    entity: GeoRouter,
    new_entity: NewGeoRouter,
    update_entity: UpdateGeoRouter,
);

// --- Manual handlers for the rule sub-resource ---

use crate::iiz::models::flows::{GeoRouterRule, NewGeoRouterRule, UpdateGeoRouterRule};

/// List rules belonging to a specific geo router.
///
/// GET `/flows/geo-routers/{router_id}/rules?page=1&per_page=25`
pub async fn list_rules(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<GeoRouterRule>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::geo_router_rules::dsl::*;

    let total: i64 = geo_router_rules
        .filter(router_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<GeoRouterRule> = geo_router_rules
        .filter(router_id.eq(parent_id))
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

/// Get a single rule by ID within a geo router.
///
/// GET `/flows/geo-routers/{router_id}/rules/{id}`
pub async fn get_rule(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, rule_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<GeoRouterRule>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::geo_router_rules::dsl::*;

    let item: GeoRouterRule = geo_router_rules
        .filter(router_id.eq(parent_id))
        .filter(id.eq(rule_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new rule in a geo router.
///
/// POST `/flows/geo-routers/{router_id}/rules`
///
/// The `router_id` from the URL path is injected into the payload to ensure
/// consistency -- the caller does not need to include it in the JSON body.
pub async fn create_rule(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewGeoRouterRule>,
) -> Result<(axum::http::StatusCode, axum::Json<GeoRouterRule>), ApiError> {
    // Override router_id from URL path for consistency
    payload.router_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: GeoRouterRule =
        diesel::insert_into(crate::iiz::schema::iiz::geo_router_rules::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update a rule within a geo router.
///
/// PUT `/flows/geo-routers/{router_id}/rules/{id}`
pub async fn update_rule(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, rule_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateGeoRouterRule>,
) -> Result<axum::Json<GeoRouterRule>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::geo_router_rules::dsl::*;

    let item: GeoRouterRule = diesel::update(
        geo_router_rules
            .filter(router_id.eq(parent_id))
            .filter(id.eq(rule_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete a rule from a geo router.
///
/// DELETE `/flows/geo-routers/{router_id}/rules/{id}`
pub async fn delete_rule(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, rule_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::geo_router_rules::dsl::*;

    diesel::update(
        geo_router_rules
            .filter(router_id.eq(parent_id))
            .filter(id.eq(rule_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
