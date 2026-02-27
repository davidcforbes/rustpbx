//! Handlers for `iiz.fax_records` -- list, get, create, soft-delete (no update).
//!
//! Fax records are append-only communication records with no Update struct.
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
use crate::iiz::models::communication::{FaxRecord, NewFaxRecord};

/// Paginated list of fax records, ordered by `created_at` descending.
///
/// GET `/activities/fax?page=1&per_page=25`
pub async fn list(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<FaxRecord>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::fax_records::dsl::created_at;

    let total: i64 = crate::iiz::schema::iiz::fax_records::table
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<FaxRecord> = crate::iiz::schema::iiz::fax_records::table
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

/// Get a single fax record by UUID.
///
/// GET `/activities/fax/{id}`
pub async fn get(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(resource_id): Path<Uuid>,
) -> Result<axum::Json<FaxRecord>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: FaxRecord = crate::iiz::schema::iiz::fax_records::table
        .find(resource_id)
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new fax record.
///
/// POST `/activities/fax`
pub async fn create(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    axum::Json(payload): axum::Json<NewFaxRecord>,
) -> Result<(axum::http::StatusCode, axum::Json<FaxRecord>), ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: FaxRecord = diesel::insert_into(crate::iiz::schema::iiz::fax_records::table)
        .values(&payload)
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Soft-delete a fax record (sets `deleted_at`).
///
/// DELETE `/activities/fax/{id}`
pub async fn delete(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(resource_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::fax_records::dsl::deleted_at;

    diesel::update(crate::iiz::schema::iiz::fax_records::table.find(resource_id))
        .set(deleted_at.eq(Some(chrono::Utc::now())))
        .execute(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
