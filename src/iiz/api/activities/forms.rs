//! Handlers for `iiz.form_records` -- list, get, create, soft-delete (no update).
//!
//! Form records are created when external form submissions are received.
//! They are append-only with no Update struct. Soft-delete is supported
//! via `deleted_at` timestamp.

use axum::extract::{Path, Query};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::iiz::api::auth::AuthContext;
use crate::iiz::api::error::ApiError;
use crate::iiz::api::middleware::get_tenant_conn;
use crate::iiz::api::pagination::{ListParams, ListResponse, PaginationMeta};
use crate::iiz::api::IizState;
use crate::iiz::models::communication::{FormRecord, NewFormRecord};

/// Paginated list of form records, ordered by `created_at` descending.
///
/// GET `/activities/forms?page=1&per_page=25`
pub async fn list(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<FormRecord>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::form_records::dsl::created_at;

    let total: i64 = crate::iiz::schema::iiz::form_records::table
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<FormRecord> = crate::iiz::schema::iiz::form_records::table
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

/// Get a single form record by UUID.
///
/// GET `/activities/forms/{id}`
pub async fn get(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(resource_id): Path<Uuid>,
) -> Result<axum::Json<FormRecord>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: FormRecord = crate::iiz::schema::iiz::form_records::table
        .find(resource_id)
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new form record.
///
/// POST `/activities/forms`
pub async fn create(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    axum::Json(payload): axum::Json<NewFormRecord>,
) -> Result<(axum::http::StatusCode, axum::Json<FormRecord>), ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: FormRecord = diesel::insert_into(crate::iiz::schema::iiz::form_records::table)
        .values(&payload)
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Soft-delete a form record (sets `deleted_at`).
///
/// DELETE `/activities/forms/{id}`
pub async fn delete(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(resource_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::form_records::dsl::deleted_at;

    diesel::update(crate::iiz::schema::iiz::form_records::table.find(resource_id))
        .set(deleted_at.eq(Some(chrono::Utc::now())))
        .execute(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
