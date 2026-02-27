//! CRUD handlers for `iiz.business_info` and its `authorized_contacts` sub-resource.
//!
//! The top-level business_info CRUD uses the `crud_handlers!` macro.
//! The authorized_contacts sub-resource requires manual handlers because:
//! - Contacts are scoped under a parent business_info via `/business-info/{info_id}/contacts`
//! - Path extractors need a parent_id + contact_id tuple for single-resource routes
//!
//! All handlers use RLS-scoped connections for account_id filtering.

// --- Standard CRUD for business_info via macro ---
//
// The macro expands module-level `use` statements for axum extractors,
// diesel prelude, uuid, and all iiz API plumbing. The manual contact handlers
// below reuse those imports.

use crate::iiz::models::trust_center::{BusinessInfo, NewBusinessInfo, UpdateBusinessInfo};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::business_info,
    entity: BusinessInfo,
    new_entity: NewBusinessInfo,
    update_entity: UpdateBusinessInfo,
);

// --- Manual handlers for the authorized_contacts sub-resource ---

use crate::iiz::models::trust_center::{AuthorizedContact, NewAuthorizedContact, UpdateAuthorizedContact};

/// List authorized contacts belonging to a specific business info record.
///
/// GET `/trust-center/business-info/{info_id}/contacts?page=1&per_page=25`
pub async fn list_contacts(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<AuthorizedContact>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::authorized_contacts::dsl::*;

    let total: i64 = authorized_contacts
        .filter(business_info_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<AuthorizedContact> = authorized_contacts
        .filter(business_info_id.eq(parent_id))
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

/// Get a single authorized contact by ID within a business info record.
///
/// GET `/trust-center/business-info/{info_id}/contacts/{id}`
pub async fn get_contact(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<AuthorizedContact>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::authorized_contacts::dsl::*;

    let item: AuthorizedContact = authorized_contacts
        .filter(business_info_id.eq(parent_id))
        .filter(id.eq(child_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new authorized contact under a business info record.
///
/// POST `/trust-center/business-info/{info_id}/contacts`
///
/// The `business_info_id` from the URL path is injected into the payload to ensure
/// consistency -- the caller does not need to include it in the JSON body.
pub async fn create_contact(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewAuthorizedContact>,
) -> Result<(axum::http::StatusCode, axum::Json<AuthorizedContact>), ApiError> {
    // Override business_info_id from URL path for consistency
    payload.business_info_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: AuthorizedContact =
        diesel::insert_into(crate::iiz::schema::iiz::authorized_contacts::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update an authorized contact within a business info record.
///
/// PUT `/trust-center/business-info/{info_id}/contacts/{id}`
pub async fn update_contact(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateAuthorizedContact>,
) -> Result<axum::Json<AuthorizedContact>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::authorized_contacts::dsl::*;

    let item: AuthorizedContact = diesel::update(
        authorized_contacts
            .filter(business_info_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete an authorized contact from a business info record.
///
/// DELETE `/trust-center/business-info/{info_id}/contacts/{id}`
pub async fn delete_contact(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::authorized_contacts::dsl::*;

    diesel::update(
        authorized_contacts
            .filter(business_info_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
