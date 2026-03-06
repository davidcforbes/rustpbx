//! Handlers for `iiz.call_records` and sub-resources (flow events, transcription,
//! summaries, annotations, tags, keyword hits, visitor sessions).
//!
//! Call records are READ-ONLY from the API -- they are created by the call
//! processing pipeline. The composite PK `(id, started_at)` means we use
//! `.filter(id.eq(...))` instead of `.find()` for single-record lookups.
//!
//! Sub-resource handlers follow the parent+child pattern: the parent `call_id`
//! comes from the URL path and is used to filter the child table.

use std::collections::HashMap;

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

use crate::iiz::models::activities::{CallRecord, CallRecordListItem};

/// Paginated list of call records with enriched data from related tables.
///
/// Batch-enrichment pattern: fetch base records first, then batch-lookup
/// tracking_sources, users, tracking_numbers, receiving_numbers,
/// call_annotations, and call_tags+tags using `WHERE id IN (...)`.
///
/// GET `/activities/calls?page=1&per_page=25&sort=started_at:desc`
pub async fn list(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<CallRecordListItem>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::call_records::dsl::*;

    // 1. Count total
    let total: i64 = call_records
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    // 2. Fetch base records with optional sort
    let mut query = call_records.into_boxed();
    match params.sort.as_deref() {
        Some("caller_phone:asc") => { query = query.order(caller_phone.asc()); }
        Some("caller_phone:desc") => { query = query.order(caller_phone.desc()); }
        Some("status:asc") => { query = query.order(status.asc()); }
        Some("status:desc") => { query = query.order(status.desc()); }
        Some("duration_secs:asc") => { query = query.order(duration_secs.asc()); }
        Some("duration_secs:desc") => { query = query.order(duration_secs.desc()); }
        Some("started_at:asc") => { query = query.order(started_at.asc()); }
        _ => { query = query.order(started_at.desc()); }
    }

    let base_items: Vec<CallRecord> = query
        .offset(offset)
        .limit(limit)
        .load(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    if base_items.is_empty() {
        let meta = PaginationMeta::new(params.page.max(1), limit, total);
        return Ok(axum::Json(ListResponse { pagination: meta, items: vec![] }));
    }

    // Collect FKs for batch lookups
    let call_ids: Vec<Uuid> = base_items.iter().map(|c| c.id).collect();
    let source_ids: Vec<Uuid> = base_items.iter().filter_map(|c| c.source_id).collect();
    let agent_ids: Vec<Uuid> = base_items.iter().filter_map(|c| c.agent_id).collect();
    let source_number_ids: Vec<Uuid> = base_items.iter().filter_map(|c| c.source_number_id).collect();

    // 3. Batch fetch tracking_sources for source_type
    let source_types: HashMap<Uuid, String> = if !source_ids.is_empty() {
        use crate::iiz::schema::iiz::tracking_sources::dsl as ts;
        let rows: Vec<(Uuid, Option<String>)> = ts::tracking_sources
            .filter(ts::id.eq_any(&source_ids))
            .select((ts::id, ts::source_type))
            .load(&mut *conn)
            .await
            .unwrap_or_default();
        rows.into_iter().filter_map(|(k, v)| v.map(|v| (k, v))).collect()
    } else {
        HashMap::new()
    };

    // 4. Batch fetch users for agent initials + avatar_color
    let agent_info: HashMap<Uuid, (Option<String>, Option<String>)> = if !agent_ids.is_empty() {
        use crate::iiz::schema::iiz::users::dsl as u;
        let rows: Vec<(Uuid, Option<String>, Option<String>)> = u::users
            .filter(u::id.eq_any(&agent_ids))
            .select((u::id, u::initials, u::avatar_color))
            .load(&mut *conn)
            .await
            .unwrap_or_default();
        rows.into_iter().map(|(k, ini, clr)| (k, (ini, clr))).collect()
    } else {
        HashMap::new()
    };

    // 5. Batch fetch tracking_numbers for number + routing_description + receiving_number_id
    let tn_info: HashMap<Uuid, (String, Option<String>, Option<Uuid>)> = if !source_number_ids.is_empty() {
        use crate::iiz::schema::iiz::tracking_numbers::dsl as tn;
        let rows: Vec<(Uuid, String, Option<String>, Option<Uuid>)> = tn::tracking_numbers
            .filter(tn::id.eq_any(&source_number_ids))
            .select((tn::id, tn::number, tn::routing_description, tn::receiving_number_id))
            .load(&mut *conn)
            .await
            .unwrap_or_default();
        rows.into_iter().map(|(k, num, rd, rn)| (k, (num, rd, rn))).collect()
    } else {
        HashMap::new()
    };

    // 6. Batch fetch receiving_numbers for the actual phone number
    let recv_number_ids: Vec<Uuid> = tn_info.values().filter_map(|(_, _, rn)| *rn).collect();
    let recv_numbers: HashMap<Uuid, String> = if !recv_number_ids.is_empty() {
        use crate::iiz::schema::iiz::receiving_numbers::dsl as rn;
        let rows: Vec<(Uuid, String)> = rn::receiving_numbers
            .filter(rn::id.eq_any(&recv_number_ids))
            .select((rn::id, rn::number))
            .load(&mut *conn)
            .await
            .unwrap_or_default();
        rows.into_iter().collect()
    } else {
        HashMap::new()
    };

    // 7. Batch fetch call_annotations for category + score
    let annotations: HashMap<Uuid, (Option<String>, Option<i32>)> = {
        use crate::iiz::schema::iiz::call_annotations::dsl as ca;
        let rows: Vec<(Uuid, Option<String>, Option<i32>)> = ca::call_annotations
            .filter(ca::call_id.eq_any(&call_ids))
            .select((ca::call_id, ca::category, ca::score))
            .load(&mut *conn)
            .await
            .unwrap_or_default();
        rows.into_iter().map(|(k, cat, sc)| (k, (cat, sc))).collect()
    };

    // 8. Batch fetch tags (call_tags JOIN tags)
    let tag_map: HashMap<Uuid, Vec<String>> = {
        use crate::iiz::schema::iiz::call_tags::dsl as ct;
        use crate::iiz::schema::iiz::tags::dsl as tg;
        let rows: Vec<(Uuid, String)> = ct::call_tags
            .inner_join(tg::tags.on(tg::id.eq(ct::tag_id)))
            .filter(ct::call_id.eq_any(&call_ids))
            .filter(ct::deleted_at.is_null())
            .select((ct::call_id, tg::name))
            .load(&mut *conn)
            .await
            .unwrap_or_default();
        let mut map: HashMap<Uuid, Vec<String>> = HashMap::new();
        for (cid, name) in rows {
            map.entry(cid).or_default().push(name);
        }
        map
    };

    // 9. Assemble enriched items
    let items: Vec<CallRecordListItem> = base_items
        .into_iter()
        .map(|c| {
            let src_type = c.source_id.and_then(|sid| source_types.get(&sid).cloned());
            let (agent_ini, agent_clr) = c.agent_id
                .and_then(|aid| agent_info.get(&aid).cloned())
                .unwrap_or((None, None));
            let (tn_number, tn_routing, tn_recv_id) = c.source_number_id
                .and_then(|tnid| tn_info.get(&tnid).cloned())
                .unwrap_or((String::new(), None, None));
            let recv_num = tn_recv_id.and_then(|rid| recv_numbers.get(&rid).cloned());
            let (ann_cat, ann_score) = annotations.get(&c.id).cloned().unwrap_or((None, None));
            let call_tags_vec = tag_map.get(&c.id).cloned().unwrap_or_default();

            CallRecordListItem {
                id: c.id,
                call_id: c.call_id,
                caller_phone: c.caller_phone,
                callee_phone: c.callee_phone,
                direction: c.direction,
                status: c.status,
                started_at: c.started_at,
                answered_at: c.answered_at,
                ended_at: c.ended_at,
                duration_secs: c.duration_secs,
                ring_duration_secs: c.ring_duration_secs,
                hold_duration_secs: c.hold_duration_secs,
                has_audio: c.has_audio,
                is_first_time_caller: c.is_first_time_caller,
                location: c.location,
                recording_url: c.recording_url,
                source_name: c.source_name,
                agent_name: c.agent_name,
                queue_name: c.queue_name,
                source_type: src_type,
                tracking_number: if tn_number.is_empty() { None } else { Some(tn_number) },
                routing_description: tn_routing,
                receiving_number: recv_num,
                agent_initials: agent_ini,
                agent_avatar_color: agent_clr,
                annotation_category: ann_cat,
                annotation_score: ann_score,
                tags: call_tags_vec,
            }
        })
        .collect();

    let meta = PaginationMeta::new(params.page.max(1), limit, total);
    Ok(axum::Json(ListResponse { pagination: meta, items }))
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
