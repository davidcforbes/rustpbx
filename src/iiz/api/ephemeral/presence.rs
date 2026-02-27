//! CRUD handlers for presence (UNLOGGED table, TEXT primary key).
//!
//! Manual handlers because the primary key is `identity` (Text), not UUID.
//! The `crud_handlers!` macro assumes `Path<Uuid>` which does not apply here.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::iiz::api::auth::AuthContext;
use crate::iiz::api::error::ApiError;
use crate::iiz::api::middleware::get_tenant_conn;
use crate::iiz::api::pagination::{ListParams, ListResponse, PaginationMeta};
use crate::iiz::api::IizState;
use crate::iiz::models::ephemeral::{NewPresence, Presence, UpdatePresence};
use crate::iiz::schema::iiz::presence;

pub async fn list(
    State(state): State<IizState>,
    auth: AuthContext,
    Query(params): Query<ListParams>,
) -> Result<Json<ListResponse<Presence>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    let total: i64 = presence::table
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<Presence> = presence::table
        .order(presence::last_updated.desc())
        .offset(offset)
        .limit(limit)
        .load(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let meta = PaginationMeta::new(params.page.max(1), limit, total);
    Ok(Json(ListResponse {
        pagination: meta,
        items,
    }))
}

pub async fn get(
    State(state): State<IizState>,
    auth: AuthContext,
    Path(identity): Path<String>,
) -> Result<Json<Presence>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let item: Presence = presence::table
        .find(&identity)
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(item))
}

pub async fn create(
    State(state): State<IizState>,
    auth: AuthContext,
    Json(payload): Json<NewPresence>,
) -> Result<(StatusCode, Json<Presence>), ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let item: Presence = diesel::insert_into(presence::table)
        .values(&payload)
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn update(
    State(state): State<IizState>,
    auth: AuthContext,
    Path(identity): Path<String>,
    Json(payload): Json<UpdatePresence>,
) -> Result<Json<Presence>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let item: Presence = diesel::update(presence::table.find(&identity))
        .set(&payload)
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(item))
}

pub async fn delete(
    State(state): State<IizState>,
    auth: AuthContext,
    Path(identity): Path<String>,
) -> Result<StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    diesel::update(presence::table.find(&identity))
        .set(presence::deleted_at.eq(Some(chrono::Utc::now())))
        .execute(&mut *conn)
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}
