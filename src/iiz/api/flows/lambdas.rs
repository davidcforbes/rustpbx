//! CRUD handlers for `iiz.lambdas` and its `lambda_env_vars` sub-resource.
//!
//! The top-level lambdas CRUD uses the `crud_handlers!` macro.
//! The env var sub-resource requires manual handlers because:
//! - Env vars are scoped under a parent lambda via `/lambdas/{lambda_id}/env-vars`
//! - Path extractors need a parent_id + child_id tuple for single-resource routes
//!
//! All handlers use RLS-scoped connections for account_id filtering.

// --- Standard CRUD for lambdas via macro ---
//
// The macro expands module-level `use` statements for axum extractors,
// diesel prelude, uuid, and all iiz API plumbing. The manual child handlers
// below reuse those imports.

use crate::iiz::models::automations::{Lambda, NewLambda, UpdateLambda};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::lambdas,
    entity: Lambda,
    new_entity: NewLambda,
    update_entity: UpdateLambda,
);

// --- Manual handlers for the lambda_env_vars sub-resource ---

use crate::iiz::models::automations::{LambdaEnvVar, NewLambdaEnvVar, UpdateLambdaEnvVar};

/// List env vars belonging to a specific lambda.
///
/// GET `/flows/lambdas/{lambda_id}/env-vars?page=1&per_page=25`
pub async fn list_env_vars(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<LambdaEnvVar>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::lambda_env_vars::dsl::*;

    let total: i64 = lambda_env_vars
        .filter(lambda_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<LambdaEnvVar> = lambda_env_vars
        .filter(lambda_id.eq(parent_id))
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

/// Get a single env var by ID within a lambda.
///
/// GET `/flows/lambdas/{lambda_id}/env-vars/{id}`
pub async fn get_env_var(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<LambdaEnvVar>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::lambda_env_vars::dsl::*;

    let item: LambdaEnvVar = lambda_env_vars
        .filter(lambda_id.eq(parent_id))
        .filter(id.eq(child_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new env var for a lambda.
///
/// POST `/flows/lambdas/{lambda_id}/env-vars`
pub async fn create_env_var(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewLambdaEnvVar>,
) -> Result<(axum::http::StatusCode, axum::Json<LambdaEnvVar>), ApiError> {
    payload.lambda_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: LambdaEnvVar =
        diesel::insert_into(crate::iiz::schema::iiz::lambda_env_vars::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update an env var within a lambda.
///
/// PUT `/flows/lambdas/{lambda_id}/env-vars/{id}`
pub async fn update_env_var(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateLambdaEnvVar>,
) -> Result<axum::Json<LambdaEnvVar>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::lambda_env_vars::dsl::*;

    let item: LambdaEnvVar = diesel::update(
        lambda_env_vars
            .filter(lambda_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete an env var from a lambda.
///
/// DELETE `/flows/lambdas/{lambda_id}/env-vars/{id}`
pub async fn delete_env_var(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::lambda_env_vars::dsl::*;

    diesel::update(
        lambda_env_vars
            .filter(lambda_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
