//! CRUD handlers for `iiz.contact_lists` and its `contact_list_members` sub-resource.
//!
//! The top-level contact_lists CRUD uses the `crud_handlers!` macro.
//! The member sub-resource requires manual handlers because:
//! - Members are scoped under a parent list via `/lists/{list_id}/members`
//! - Path extractors need a parent_id + member_id tuple for single-resource routes
//!
//! All handlers use RLS-scoped connections for account_id filtering.

// --- Standard CRUD for contact_lists via macro ---
//
// The macro expands module-level `use` statements for axum extractors,
// diesel prelude, uuid, and all iiz API plumbing. The manual member handlers
// below reuse those imports.

use crate::iiz::models::contacts::{ContactList, NewContactList, UpdateContactList};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::contact_lists,
    entity: ContactList,
    new_entity: NewContactList,
    update_entity: UpdateContactList,
);

// --- Manual handlers for the member sub-resource ---

use crate::iiz::models::contacts::{
    ContactListMember, NewContactListMember, UpdateContactListMember,
};

/// List members belonging to a specific contact list.
///
/// GET `/contacts/lists/{list_id}/members?page=1&per_page=25`
pub async fn list_members(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<ContactListMember>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::contact_list_members::dsl::*;

    let total: i64 = contact_list_members
        .filter(list_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<ContactListMember> = contact_list_members
        .filter(list_id.eq(parent_id))
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

/// Get a single member by ID within a contact list.
///
/// GET `/contacts/lists/{list_id}/members/{id}`
pub async fn get_member(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, member_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<ContactListMember>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::contact_list_members::dsl::*;

    let item: ContactListMember = contact_list_members
        .filter(list_id.eq(parent_id))
        .filter(id.eq(member_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new member in a contact list.
///
/// POST `/contacts/lists/{list_id}/members`
///
/// The `list_id` from the URL path is injected into the payload to ensure
/// consistency -- the caller does not need to include it in the JSON body.
pub async fn create_member(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewContactListMember>,
) -> Result<(axum::http::StatusCode, axum::Json<ContactListMember>), ApiError> {
    // Override list_id from URL path for consistency
    payload.list_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: ContactListMember =
        diesel::insert_into(crate::iiz::schema::iiz::contact_list_members::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update a member within a contact list.
///
/// PUT `/contacts/lists/{list_id}/members/{id}`
pub async fn update_member(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, member_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateContactListMember>,
) -> Result<axum::Json<ContactListMember>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::contact_list_members::dsl::*;

    let item: ContactListMember = diesel::update(
        contact_list_members
            .filter(list_id.eq(parent_id))
            .filter(id.eq(member_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete a member from a contact list.
///
/// DELETE `/contacts/lists/{list_id}/members/{id}`
pub async fn delete_member(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, member_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::contact_list_members::dsl::*;

    diesel::update(
        contact_list_members
            .filter(list_id.eq(parent_id))
            .filter(id.eq(member_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
