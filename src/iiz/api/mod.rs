//! 4iiz REST API -- axum router, shared state, and section assembly.
//!
//! This module provides the foundational plumbing for the 4iiz call tracking
//! platform's REST API:
//!
//! - **`IizState`** -- shared state holding connection pools and JWT config
//! - **`router()`** -- builds the `/api/v1` router tree with section nesting
//! - **`error`** -- `ApiError` enum with `IntoResponse`
//! - **`pagination`** -- `ListParams`, `PaginationMeta`, `ListResponse`
//! - **`auth`** -- JWT auth extractor with dev-mode header bypass
//! - **`middleware`** -- tenant-scoped database connection helpers
//! - **`crud`** -- `crud_handlers!` macro for standard CRUD endpoints

use std::sync::Arc;

use axum::Router;

use crate::iiz::pool::IizPools;

pub mod activities;
pub mod ai_tools;
pub mod auth;
pub mod contacts;
pub mod crud;
pub mod error;
pub mod flows;
pub mod middleware;
pub mod numbers;
pub mod pagination;
pub mod tags;

/// Shared state for all iiz API handlers.
///
/// Passed to the axum router via `.with_state()`. Handlers extract it with
/// `State<IizState>` and the auth extractor uses `FromRef<S>` to access it.
#[derive(Clone)]
pub struct IizState {
    /// Segregated Diesel-async connection pools.
    pub pools: Arc<IizPools>,
    /// HMAC secret for JWT signing/verification.
    /// Empty = dev mode (accept X-Account-Id / X-User-Id headers).
    pub jwt_secret: Vec<u8>,
}

/// Build the complete `/api/v1` router tree.
///
/// Section routers will be nested here as they are implemented in later phases:
/// - `/api/v1/activities`
/// - `/api/v1/contacts`
/// - `/api/v1/numbers`
/// - `/api/v1/flows`
/// - `/api/v1/ai-tools`
/// - `/api/v1/reports`
/// - `/api/v1/trust-center`
pub fn router(state: IizState) -> Router {
    let api = Router::new()
        // -- Activities section (Phase F6.3) --
        .nest("/activities", activities::router())
        // -- Contacts section (Phase F1.3) --
        .nest("/contacts", contacts::router())
        // -- Numbers section (Phase F2.2) --
        .nest("/numbers", numbers::router())
        // -- Flows section (Phase F3.2) --
        .nest("/flows", flows::router())
        // -- AI Tools section (Phase F5.2) --
        .nest("/ai-tools", ai_tools::router())
        // -- Tags (cross-cutting, used by multiple sections) --
        .route(
            "/tags",
            axum::routing::get(tags::list).post(tags::create),
        )
        .route(
            "/tags/{id}",
            axum::routing::get(tags::get)
                .put(tags::update)
                .delete(tags::delete),
        )
        // Section routers will be nested here in later phases:
        // .nest("/reports", reports::router())
        // .nest("/trust-center", trust_center::router())
        .with_state(state);

    Router::new().nest("/api/v1", api)
}
