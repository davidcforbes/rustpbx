//! `crud_handlers!` macro for stamping standard CRUD endpoints.
//!
//! Tables that follow the standard pattern (UUID `id`, `account_id`,
//! `created_at`, `updated_at`, `deleted_at`) can use this macro to generate
//! `list`, `get`, `create`, `update`, and `delete` handler functions.
//!
//! The generated handlers use RLS-scoped connections (via `get_tenant_conn`)
//! so `account_id` filtering is handled at the database layer.

/// Generate standard CRUD handler functions for a Diesel table.
///
/// # Arguments
///
/// - `table`: Path to the Diesel `table!` macro (e.g. `crate::iiz::schema::iiz::blocked_numbers`)
/// - `entity`: The Queryable struct for reading rows
/// - `new_entity`: The Insertable struct for creating rows
/// - `update_entity`: The AsChangeset struct for patching rows
///
/// # Generated functions
///
/// - `pub async fn list(...)` -- paginated list (RLS filters account + soft-deletes)
/// - `pub async fn get(...)` -- single resource by UUID
/// - `pub async fn create(...)` -- insert new resource, returns 201
/// - `pub async fn update(...)` -- partial update by UUID
/// - `pub async fn delete(...)` -- soft delete (sets `deleted_at`), returns 204
///
/// # Example
///
/// ```ignore
/// mod blocked_numbers {
///     use crate::crud_handlers;
///     crud_handlers!(
///         table: crate::iiz::schema::iiz::blocked_numbers,
///         entity: BlockedNumber,
///         new_entity: NewBlockedNumber,
///         update_entity: UpdateBlockedNumber,
///     );
/// }
/// ```
#[macro_export]
macro_rules! crud_handlers {
    (
        table: $table:path,
        entity: $entity:ty,
        new_entity: $new_entity:ty,
        update_entity: $update_entity:ty $(,)?
    ) => {
        use axum::extract::{Path, Query};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use uuid::Uuid;
        use $crate::iiz::api::auth::AuthContext;
        use $crate::iiz::api::error::ApiError;
        use $crate::iiz::api::middleware::get_tenant_conn;
        use $crate::iiz::api::pagination::{ListParams, ListResponse, PaginationMeta};
        use $crate::iiz::api::IizState;

        pub async fn list(
            axum::extract::State(state): axum::extract::State<IizState>,
            auth: AuthContext,
            Query(params): Query<ListParams>,
        ) -> Result<axum::Json<ListResponse<$entity>>, ApiError> {
            let mut conn = get_tenant_conn(&state, &auth).await?;
            let (offset, limit) = params.normalize();

            // RLS handles account_id filtering; deleted_at filtering via RLS policy.
            let total: i64 = {
                use $table::dsl::*;
                $table::table.count().get_result(&mut *conn).await.map_err(ApiError::from)?
            };

            let items: Vec<$entity> = {
                use $table::dsl::*;
                $table::table
                    .order(created_at.desc())
                    .offset(offset)
                    .limit(limit)
                    .load(&mut *conn)
                    .await
                    .map_err(ApiError::from)?
            };

            let meta = PaginationMeta::new(params.page.max(1), limit, total);
            Ok(axum::Json(ListResponse {
                pagination: meta,
                items,
            }))
        }

        pub async fn get(
            axum::extract::State(state): axum::extract::State<IizState>,
            auth: AuthContext,
            Path(id): Path<Uuid>,
        ) -> Result<axum::Json<$entity>, ApiError> {
            let mut conn = get_tenant_conn(&state, &auth).await?;
            use $table::dsl;
            let item: $entity = dsl::$table
                .find(id)
                .first(&mut *conn)
                .await
                .map_err(ApiError::from)?;
            Ok(axum::Json(item))
        }

        pub async fn create(
            axum::extract::State(state): axum::extract::State<IizState>,
            auth: AuthContext,
            axum::Json(payload): axum::Json<$new_entity>,
        ) -> Result<(axum::http::StatusCode, axum::Json<$entity>), ApiError> {
            let mut conn = get_tenant_conn(&state, &auth).await?;
            let item: $entity = diesel::insert_into($table::table)
                .values(&payload)
                .get_result(&mut *conn)
                .await
                .map_err(ApiError::from)?;
            Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
        }

        pub async fn update(
            axum::extract::State(state): axum::extract::State<IizState>,
            auth: AuthContext,
            Path(id): Path<Uuid>,
            axum::Json(payload): axum::Json<$update_entity>,
        ) -> Result<axum::Json<$entity>, ApiError> {
            let mut conn = get_tenant_conn(&state, &auth).await?;
            use $table::dsl;
            let item: $entity = diesel::update(dsl::$table.find(id))
                .set(&payload)
                .get_result(&mut *conn)
                .await
                .map_err(ApiError::from)?;
            Ok(axum::Json(item))
        }

        pub async fn delete(
            axum::extract::State(state): axum::extract::State<IizState>,
            auth: AuthContext,
            Path(id): Path<Uuid>,
        ) -> Result<axum::http::StatusCode, ApiError> {
            let mut conn = get_tenant_conn(&state, &auth).await?;
            use $table::dsl::*;
            diesel::update($table::table.find(id))
                .set(deleted_at.eq(Some(chrono::Utc::now())))
                .execute(&mut *conn)
                .await
                .map_err(ApiError::from)?;
            Ok(axum::http::StatusCode::NO_CONTENT)
        }
    };
}
