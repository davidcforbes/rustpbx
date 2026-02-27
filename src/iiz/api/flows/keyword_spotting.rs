//! CRUD handlers for `iiz.keyword_spotting_configs` and its `keyword_spotting_keywords` /
//! `keyword_spotting_numbers` sub-resources.
//!
//! The top-level keyword spotting config CRUD uses the `crud_handlers!` macro.
//! The keyword and number sub-resources require manual handlers because:
//! - They are scoped under a parent config via `/keyword-spotting/{config_id}/keywords` and `/numbers`
//! - Path extractors need a parent_id + child_id tuple for single-resource routes
//!
//! All handlers use RLS-scoped connections for account_id filtering.

// --- Standard CRUD for keyword_spotting_configs via macro ---
//
// The macro expands module-level `use` statements for axum extractors,
// diesel prelude, uuid, and all iiz API plumbing. The manual child handlers
// below reuse those imports.

use crate::iiz::models::engagement::{KeywordSpottingConfig, NewKeywordSpottingConfig, UpdateKeywordSpottingConfig};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::keyword_spotting_configs,
    entity: KeywordSpottingConfig,
    new_entity: NewKeywordSpottingConfig,
    update_entity: UpdateKeywordSpottingConfig,
);

// --- Manual handlers for the keyword_spotting_keywords sub-resource ---

use crate::iiz::models::engagement::{KeywordSpottingKeyword, NewKeywordSpottingKeyword, UpdateKeywordSpottingKeyword};

/// List keywords belonging to a specific keyword spotting config.
///
/// GET `/flows/keyword-spotting/{config_id}/keywords?page=1&per_page=25`
pub async fn list_keywords(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<KeywordSpottingKeyword>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::keyword_spotting_keywords::dsl::*;

    let total: i64 = keyword_spotting_keywords
        .filter(config_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<KeywordSpottingKeyword> = keyword_spotting_keywords
        .filter(config_id.eq(parent_id))
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

/// Get a single keyword by ID within a keyword spotting config.
///
/// GET `/flows/keyword-spotting/{config_id}/keywords/{id}`
pub async fn get_keyword(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<KeywordSpottingKeyword>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::keyword_spotting_keywords::dsl::*;

    let item: KeywordSpottingKeyword = keyword_spotting_keywords
        .filter(config_id.eq(parent_id))
        .filter(id.eq(child_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new keyword in a keyword spotting config.
///
/// POST `/flows/keyword-spotting/{config_id}/keywords`
pub async fn create_keyword(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewKeywordSpottingKeyword>,
) -> Result<(axum::http::StatusCode, axum::Json<KeywordSpottingKeyword>), ApiError> {
    payload.config_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: KeywordSpottingKeyword =
        diesel::insert_into(crate::iiz::schema::iiz::keyword_spotting_keywords::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update a keyword within a keyword spotting config.
///
/// PUT `/flows/keyword-spotting/{config_id}/keywords/{id}`
pub async fn update_keyword(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateKeywordSpottingKeyword>,
) -> Result<axum::Json<KeywordSpottingKeyword>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::keyword_spotting_keywords::dsl::*;

    let item: KeywordSpottingKeyword = diesel::update(
        keyword_spotting_keywords
            .filter(config_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete a keyword from a keyword spotting config.
///
/// DELETE `/flows/keyword-spotting/{config_id}/keywords/{id}`
pub async fn delete_keyword(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::keyword_spotting_keywords::dsl::*;

    diesel::update(
        keyword_spotting_keywords
            .filter(config_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// --- Manual handlers for the keyword_spotting_numbers sub-resource ---

use crate::iiz::models::engagement::{KeywordSpottingNumber, NewKeywordSpottingNumber, UpdateKeywordSpottingNumber};

/// List numbers belonging to a specific keyword spotting config.
///
/// GET `/flows/keyword-spotting/{config_id}/numbers?page=1&per_page=25`
pub async fn list_numbers(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<KeywordSpottingNumber>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::keyword_spotting_numbers::dsl::*;

    let total: i64 = keyword_spotting_numbers
        .filter(config_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<KeywordSpottingNumber> = keyword_spotting_numbers
        .filter(config_id.eq(parent_id))
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

/// Get a single number by ID within a keyword spotting config.
///
/// GET `/flows/keyword-spotting/{config_id}/numbers/{id}`
pub async fn get_number(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<KeywordSpottingNumber>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::keyword_spotting_numbers::dsl::*;

    let item: KeywordSpottingNumber = keyword_spotting_numbers
        .filter(config_id.eq(parent_id))
        .filter(id.eq(child_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new number in a keyword spotting config.
///
/// POST `/flows/keyword-spotting/{config_id}/numbers`
pub async fn create_number(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewKeywordSpottingNumber>,
) -> Result<(axum::http::StatusCode, axum::Json<KeywordSpottingNumber>), ApiError> {
    payload.config_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: KeywordSpottingNumber =
        diesel::insert_into(crate::iiz::schema::iiz::keyword_spotting_numbers::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update a number within a keyword spotting config.
///
/// PUT `/flows/keyword-spotting/{config_id}/numbers/{id}`
pub async fn update_number(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateKeywordSpottingNumber>,
) -> Result<axum::Json<KeywordSpottingNumber>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::keyword_spotting_numbers::dsl::*;

    let item: KeywordSpottingNumber = diesel::update(
        keyword_spotting_numbers
            .filter(config_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete a number from a keyword spotting config.
///
/// DELETE `/flows/keyword-spotting/{config_id}/numbers/{id}`
pub async fn delete_number(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::keyword_spotting_numbers::dsl::*;

    diesel::update(
        keyword_spotting_numbers
            .filter(config_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
