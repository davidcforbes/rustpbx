//! Handlers for `iiz.video_records` -- list, get, create, soft-delete (no update).
//!
//! Video records are append-only communication records with no Update struct.
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
use crate::iiz::models::communication::{NewVideoRecord, VideoRecord};

/// Paginated list of video records, ordered by `created_at` descending.
///
/// GET `/activities/video?page=1&per_page=25`
pub async fn list(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<VideoRecord>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::video_records::dsl::created_at;

    let total: i64 = crate::iiz::schema::iiz::video_records::table
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<VideoRecord> = crate::iiz::schema::iiz::video_records::table
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

/// Get a single video record by UUID.
///
/// GET `/activities/video/{id}`
pub async fn get(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(resource_id): Path<Uuid>,
) -> Result<axum::Json<VideoRecord>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: VideoRecord = crate::iiz::schema::iiz::video_records::table
        .find(resource_id)
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new video record.
///
/// POST `/activities/video`
pub async fn create(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    axum::Json(payload): axum::Json<NewVideoRecord>,
) -> Result<(axum::http::StatusCode, axum::Json<VideoRecord>), ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: VideoRecord = diesel::insert_into(crate::iiz::schema::iiz::video_records::table)
        .values(&payload)
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Soft-delete a video record (sets `deleted_at`).
///
/// DELETE `/activities/video/{id}`
pub async fn delete(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(resource_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::video_records::dsl::deleted_at;

    diesel::update(crate::iiz::schema::iiz::video_records::table.find(resource_id))
        .set(deleted_at.eq(Some(chrono::Utc::now())))
        .execute(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
