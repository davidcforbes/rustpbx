//! Handlers for `iiz.call_records` and sub-resources (flow events, transcription,
//! summaries, annotations, tags, keyword hits, visitor sessions).
//!
//! Call records are READ-ONLY from the API -- they are created by the call
//! processing pipeline. The composite PK `(id, started_at)` means we use
//! `.filter(id.eq(...))` instead of `.find()` for single-record lookups.
//!
//! Sub-resource handlers follow the parent+child pattern: the parent `call_id`
//! comes from the URL path and is used to filter the child table.

use axum::extract::{Path, Query};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::iiz::api::auth::AuthContext;
use crate::iiz::api::error::ApiError;
use crate::iiz::api::middleware::get_tenant_conn;
use crate::iiz::api::pagination::{ListParams, ListResponse, PaginationMeta};
use crate::iiz::api::IizState;

// =====================================================================
// call_records — list + get (read-only)
// =====================================================================

use crate::iiz::models::activities::CallRecord;

/// Paginated list of call records, ordered by `started_at` descending.
///
/// GET `/activities/calls?page=1&per_page=25`
pub async fn list(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<CallRecord>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::call_records::dsl::*;

    let total: i64 = call_records
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<CallRecord> = call_records
        .order(started_at.desc())
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

/// Get a single call record by UUID.
///
/// GET `/activities/calls/{id}`
///
/// Uses `.filter(id.eq(...))` instead of `.find()` because call_records has a
/// composite primary key `(id, started_at)`.
pub async fn get(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(call_id_param): Path<Uuid>,
) -> Result<axum::Json<CallRecord>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::call_records::dsl::*;

    let item: CallRecord = call_records
        .filter(id.eq(call_id_param))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

// =====================================================================
// call_flow_events — list by call_id, ordered by occurred_at ASC
// =====================================================================

use crate::iiz::models::activities::CallFlowEvent;

/// List flow events for a call, in chronological order.
///
/// GET `/activities/calls/{call_id}/flow?page=1&per_page=25`
pub async fn list_flow_events(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<CallFlowEvent>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::call_flow_events::dsl::*;

    let total: i64 = call_flow_events
        .filter(call_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<CallFlowEvent> = call_flow_events
        .filter(call_id.eq(parent_id))
        .order(occurred_at.asc())
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

// =====================================================================
// call_transcription_segments — list by call_id, ordered by segment_index ASC
// =====================================================================

use crate::iiz::models::activities::CallTranscriptionSegment;

/// List transcription segments for a call, ordered by segment index.
///
/// GET `/activities/calls/{call_id}/transcription?page=1&per_page=25`
pub async fn list_transcription_segments(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<CallTranscriptionSegment>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::call_transcription_segments::dsl::*;

    let total: i64 = call_transcription_segments
        .filter(call_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<CallTranscriptionSegment> = call_transcription_segments
        .filter(call_id.eq(parent_id))
        .order(segment_index.asc())
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

// =====================================================================
// call_ai_summaries — list by call_id, ordered by generated_at DESC
// =====================================================================

use crate::iiz::models::activities::CallAiSummary;

/// List AI summaries for a call, most recent first.
///
/// GET `/activities/calls/{call_id}/summaries?page=1&per_page=25`
pub async fn list_summaries(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<CallAiSummary>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::call_ai_summaries::dsl::*;

    let total: i64 = call_ai_summaries
        .filter(call_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<CallAiSummary> = call_ai_summaries
        .filter(call_id.eq(parent_id))
        .order(generated_at.desc())
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

// =====================================================================
// call_annotations — get single + upsert (1:1 with call_records)
// =====================================================================

use crate::iiz::models::activities::{CallAnnotation, UpdateCallAnnotation};

/// Get the annotation for a call (1:1 relationship).
///
/// GET `/activities/calls/{call_id}/annotations`
pub async fn get_annotation(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
) -> Result<axum::Json<CallAnnotation>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::call_annotations::dsl::*;

    let item: CallAnnotation = call_annotations
        .filter(call_id.eq(parent_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Update (upsert) the annotation for a call.
///
/// PUT `/activities/calls/{call_id}/annotations`
pub async fn upsert_annotation(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(payload): axum::Json<UpdateCallAnnotation>,
) -> Result<axum::Json<CallAnnotation>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::call_annotations::dsl::*;

    let item: CallAnnotation = diesel::update(
        call_annotations.filter(call_id.eq(parent_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

// =====================================================================
// call_tags — list, create, delete (scoped by call_id)
// =====================================================================

use crate::iiz::models::activities::{CallTag, NewCallTag};

/// List tags applied to a call.
///
/// GET `/activities/calls/{call_id}/tags?page=1&per_page=25`
pub async fn list_tags(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<CallTag>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::call_tags::dsl::*;

    let total: i64 = call_tags
        .filter(call_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<CallTag> = call_tags
        .filter(call_id.eq(parent_id))
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

/// Apply a tag to a call.
///
/// POST `/activities/calls/{call_id}/tags`
///
/// The `call_id` from the URL path is injected into the payload.
pub async fn create_tag(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewCallTag>,
) -> Result<(axum::http::StatusCode, axum::Json<CallTag>), ApiError> {
    payload.call_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: CallTag = diesel::insert_into(crate::iiz::schema::iiz::call_tags::table)
        .values(&payload)
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Remove a tag from a call (soft-delete).
///
/// DELETE `/activities/calls/{call_id}/tags/{id}`
pub async fn delete_tag(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::call_tags::dsl::*;

    diesel::update(
        call_tags
            .filter(call_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// =====================================================================
// call_keyword_hits — list by call_id
// =====================================================================

use crate::iiz::models::activities::CallKeywordHit;

/// List keyword hits detected in a call.
///
/// GET `/activities/calls/{call_id}/keyword-hits?page=1&per_page=25`
pub async fn list_keyword_hits(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<CallKeywordHit>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::call_keyword_hits::dsl::*;

    let total: i64 = call_keyword_hits
        .filter(call_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<CallKeywordHit> = call_keyword_hits
        .filter(call_id.eq(parent_id))
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

// =====================================================================
// call_visitor_sessions — get single (1:1 with call)
// =====================================================================

use crate::iiz::models::activities::CallVisitorSession;

/// Get the visitor session associated with a call (1:1 relationship).
///
/// GET `/activities/calls/{call_id}/visitor`
pub async fn get_visitor_session(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
) -> Result<axum::Json<CallVisitorSession>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::call_visitor_sessions::dsl::*;

    let item: CallVisitorSession = call_visitor_sessions
        .filter(call_id.eq(parent_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}
