//! Tenant context helpers for RLS-scoped database connections.
//!
//! Handlers extract `AuthContext` (via the auth extractor) and then call
//! `get_tenant_conn()` to obtain a pooled connection with the RLS context
//! variable `app.current_account_id` already set.

use crate::iiz::api::auth::AuthContext;
use crate::iiz::api::error::ApiError;
use crate::iiz::api::IizState;
use diesel_async::pooled_connection::bb8::PooledConnection;
use diesel_async::AsyncPgConnection;

/// Get a tenant-scoped connection from the `api_crud` pool.
///
/// Call this in handlers after extracting `AuthContext`. The returned
/// connection has `app.current_account_id` set for row-level security.
///
/// # Example
///
/// ```ignore
/// async fn list_items(
///     State(state): State<IizState>,
///     auth: AuthContext,
/// ) -> Result<Json<...>, ApiError> {
///     let mut conn = get_tenant_conn(&state, &auth).await?;
///     // ... use conn for queries
/// }
/// ```
pub async fn get_tenant_conn<'a>(
    state: &'a IizState,
    auth: &AuthContext,
) -> Result<PooledConnection<'a, AsyncPgConnection>, ApiError> {
    crate::iiz::pool::IizPools::set_tenant(&state.pools.api_crud, &auth.account_id)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get database connection: {}", e)))
}
