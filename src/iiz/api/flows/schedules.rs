//! CRUD handlers for `iiz.schedules` and its `schedule_holidays` sub-resource.
//!
//! The top-level schedules CRUD uses the `crud_handlers!` macro.
//! The holiday sub-resource requires manual handlers because:
//! - Holidays are scoped under a parent schedule via `/schedules/{schedule_id}/holidays`
//! - Path extractors need a parent_id + holiday_id tuple for single-resource routes
//!
//! All handlers use RLS-scoped connections for account_id filtering.

// --- Standard CRUD for schedules via macro ---
//
// The macro expands module-level `use` statements for axum extractors,
// diesel prelude, uuid, and all iiz API plumbing. The manual holiday handlers
// below reuse those imports.

use crate::iiz::models::flows::{NewSchedule, Schedule, UpdateSchedule};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::schedules,
    entity: Schedule,
    new_entity: NewSchedule,
    update_entity: UpdateSchedule,
);

// --- Manual handlers for the holiday sub-resource ---

use crate::iiz::models::flows::{NewScheduleHoliday, ScheduleHoliday, UpdateScheduleHoliday};

/// List holidays belonging to a specific schedule.
///
/// GET `/flows/schedules/{schedule_id}/holidays?page=1&per_page=25`
pub async fn list_holidays(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<ScheduleHoliday>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::schedule_holidays::dsl::*;

    let total: i64 = schedule_holidays
        .filter(schedule_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<ScheduleHoliday> = schedule_holidays
        .filter(schedule_id.eq(parent_id))
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

/// Get a single holiday by ID within a schedule.
///
/// GET `/flows/schedules/{schedule_id}/holidays/{id}`
pub async fn get_holiday(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, holiday_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<ScheduleHoliday>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::schedule_holidays::dsl::*;

    let item: ScheduleHoliday = schedule_holidays
        .filter(schedule_id.eq(parent_id))
        .filter(id.eq(holiday_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new holiday in a schedule.
///
/// POST `/flows/schedules/{schedule_id}/holidays`
///
/// The `schedule_id` from the URL path is injected into the payload to ensure
/// consistency -- the caller does not need to include it in the JSON body.
pub async fn create_holiday(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewScheduleHoliday>,
) -> Result<(axum::http::StatusCode, axum::Json<ScheduleHoliday>), ApiError> {
    // Override schedule_id from URL path for consistency
    payload.schedule_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: ScheduleHoliday =
        diesel::insert_into(crate::iiz::schema::iiz::schedule_holidays::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update a holiday within a schedule.
///
/// PUT `/flows/schedules/{schedule_id}/holidays/{id}`
pub async fn update_holiday(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, holiday_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateScheduleHoliday>,
) -> Result<axum::Json<ScheduleHoliday>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::schedule_holidays::dsl::*;

    let item: ScheduleHoliday = diesel::update(
        schedule_holidays
            .filter(schedule_id.eq(parent_id))
            .filter(id.eq(holiday_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete a holiday from a schedule.
///
/// DELETE `/flows/schedules/{schedule_id}/holidays/{id}`
pub async fn delete_holiday(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, holiday_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::schedule_holidays::dsl::*;

    diesel::update(
        schedule_holidays
            .filter(schedule_id.eq(parent_id))
            .filter(id.eq(holiday_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
