//! Handlers for `iiz.text_records` -- list, get, create, soft-delete (no update).
//!
//! Text records are append-only communication records with no Update struct.
//! Soft-delete is supported via `deleted_at` timestamp.

use axum::extract::{Path, Query};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::iiz::api::auth::AuthContext;
use crate::iiz::api::error::ApiError;
use crate::iiz::api::middleware::get_tenant_conn;
use crate::iiz::api::pagination::{ListParams, ListResponse, PaginationMeta};
use crate::iiz::api::IizState;
use crate::iiz::models::communication::{NewTextRecord, TextRecord};

/// Paginated list of text records, ordered by `created_at` descending.
///
/// GET `/activities/texts?page=1&per_page=25`
pub async fn list(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<TextRecord>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::text_records::dsl::created_at;

    let total: i64 = crate::iiz::schema::iiz::text_records::table
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<TextRecord> = crate::iiz::schema::iiz::text_records::table
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

/// Get a single text record by UUID.
///
/// GET `/activities/texts/{id}`
pub async fn get(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(resource_id): Path<Uuid>,
) -> Result<axum::Json<TextRecord>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: TextRecord = crate::iiz::schema::iiz::text_records::table
        .find(resource_id)
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new text record.
///
/// POST `/activities/texts`
pub async fn create(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    axum::Json(payload): axum::Json<NewTextRecord>,
) -> Result<(axum::http::StatusCode, axum::Json<TextRecord>), ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: TextRecord = diesel::insert_into(crate::iiz::schema::iiz::text_records::table)
        .values(&payload)
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Soft-delete a text record (sets `deleted_at`).
///
/// DELETE `/activities/texts/{id}`
pub async fn delete(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(resource_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::text_records::dsl::deleted_at;

    diesel::update(crate::iiz::schema::iiz::text_records::table.find(resource_id))
        .set(deleted_at.eq(Some(chrono::Utc::now())))
        .execute(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
