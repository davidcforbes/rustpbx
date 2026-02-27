//! CRUD handlers for `iiz.webhooks` and its `webhook_subscriptions` sub-resource.
//!
//! The top-level webhooks CRUD uses the `crud_handlers!` macro.
//! The subscription sub-resource requires manual handlers because:
//! - Subscriptions are scoped under a parent webhook via `/webhooks/{webhook_id}/subscriptions`
//! - Path extractors need a parent_id + child_id tuple for single-resource routes
//!
//! All handlers use RLS-scoped connections for account_id filtering.

// --- Standard CRUD for webhooks via macro ---
//
// The macro expands module-level `use` statements for axum extractors,
// diesel prelude, uuid, and all iiz API plumbing. The manual child handlers
// below reuse those imports.

use crate::iiz::models::automations::{NewWebhook, UpdateWebhook, Webhook};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::webhooks,
    entity: Webhook,
    new_entity: NewWebhook,
    update_entity: UpdateWebhook,
);

// --- Manual handlers for the webhook_subscriptions sub-resource ---

use crate::iiz::models::automations::{NewWebhookSubscription, UpdateWebhookSubscription, WebhookSubscription};

/// List subscriptions belonging to a specific webhook.
///
/// GET `/flows/webhooks/{webhook_id}/subscriptions?page=1&per_page=25`
pub async fn list_subscriptions(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<WebhookSubscription>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::webhook_subscriptions::dsl::*;

    let total: i64 = webhook_subscriptions
        .filter(webhook_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<WebhookSubscription> = webhook_subscriptions
        .filter(webhook_id.eq(parent_id))
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

/// Get a single subscription by ID within a webhook.
///
/// GET `/flows/webhooks/{webhook_id}/subscriptions/{id}`
pub async fn get_subscription(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<WebhookSubscription>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::webhook_subscriptions::dsl::*;

    let item: WebhookSubscription = webhook_subscriptions
        .filter(webhook_id.eq(parent_id))
        .filter(id.eq(child_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new subscription for a webhook.
///
/// POST `/flows/webhooks/{webhook_id}/subscriptions`
pub async fn create_subscription(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewWebhookSubscription>,
) -> Result<(axum::http::StatusCode, axum::Json<WebhookSubscription>), ApiError> {
    payload.webhook_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: WebhookSubscription =
        diesel::insert_into(crate::iiz::schema::iiz::webhook_subscriptions::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update a subscription within a webhook.
///
/// PUT `/flows/webhooks/{webhook_id}/subscriptions/{id}`
pub async fn update_subscription(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateWebhookSubscription>,
) -> Result<axum::Json<WebhookSubscription>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::webhook_subscriptions::dsl::*;

    let item: WebhookSubscription = diesel::update(
        webhook_subscriptions
            .filter(webhook_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete a subscription from a webhook.
///
/// DELETE `/flows/webhooks/{webhook_id}/subscriptions/{id}`
pub async fn delete_subscription(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::webhook_subscriptions::dsl::*;

    diesel::update(
        webhook_subscriptions
            .filter(webhook_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
