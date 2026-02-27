//! Number section router -- assembles CRUD routes for all number sub-resources.
//!
//! Sub-resources:
//! - `/numbers/tracking` -- tracking numbers
//! - `/numbers/sources` -- tracking sources
//! - `/numbers/receiving` -- receiving numbers
//! - `/numbers/targets` -- target numbers
//! - `/numbers/text` -- text-enabled numbers
//! - `/numbers/pools` -- number pools with nested `/pools/{pool_id}/members`
//! - `/numbers/caller-id` -- caller ID CNAM entries
//! - `/numbers/port-requests` -- number port requests
//! - `/numbers/call-settings` -- call settings profiles

use axum::routing::get;
use axum::Router;

use crate::iiz::api::IizState;

mod call_settings;
mod caller_id;
mod number_pools;
mod port_requests;
mod receiving_numbers;
mod target_numbers;
mod text_numbers;
mod tracking_numbers;
mod tracking_sources;

/// Build the `/numbers` section router.
///
/// All routes require authentication (via `AuthContext` extractor) and use
/// RLS-scoped connections for tenant isolation.
pub fn router() -> Router<IizState> {
    Router::new()
        // --- Tracking Numbers ---
        .route(
            "/tracking",
            get(tracking_numbers::list).post(tracking_numbers::create),
        )
        .route(
            "/tracking/{id}",
            get(tracking_numbers::get)
                .put(tracking_numbers::update)
                .delete(tracking_numbers::delete),
        )
        // --- Tracking Sources ---
        .route(
            "/sources",
            get(tracking_sources::list).post(tracking_sources::create),
        )
        .route(
            "/sources/{id}",
            get(tracking_sources::get)
                .put(tracking_sources::update)
                .delete(tracking_sources::delete),
        )
        // --- Receiving Numbers ---
        .route(
            "/receiving",
            get(receiving_numbers::list).post(receiving_numbers::create),
        )
        .route(
            "/receiving/{id}",
            get(receiving_numbers::get)
                .put(receiving_numbers::update)
                .delete(receiving_numbers::delete),
        )
        // --- Target Numbers ---
        .route(
            "/targets",
            get(target_numbers::list).post(target_numbers::create),
        )
        .route(
            "/targets/{id}",
            get(target_numbers::get)
                .put(target_numbers::update)
                .delete(target_numbers::delete),
        )
        // --- Text Numbers ---
        .route(
            "/text",
            get(text_numbers::list).post(text_numbers::create),
        )
        .route(
            "/text/{id}",
            get(text_numbers::get)
                .put(text_numbers::update)
                .delete(text_numbers::delete),
        )
        // --- Number Pools + Members sub-resource ---
        .route(
            "/pools",
            get(number_pools::list).post(number_pools::create),
        )
        .route(
            "/pools/{id}",
            get(number_pools::get)
                .put(number_pools::update)
                .delete(number_pools::delete),
        )
        .route(
            "/pools/{pool_id}/members",
            get(number_pools::list_members).post(number_pools::create_member),
        )
        .route(
            "/pools/{pool_id}/members/{id}",
            get(number_pools::get_member)
                .put(number_pools::update_member)
                .delete(number_pools::delete_member),
        )
        // --- Caller ID CNAM ---
        .route(
            "/caller-id",
            get(caller_id::list).post(caller_id::create),
        )
        .route(
            "/caller-id/{id}",
            get(caller_id::get)
                .put(caller_id::update)
                .delete(caller_id::delete),
        )
        // --- Port Requests ---
        .route(
            "/port-requests",
            get(port_requests::list).post(port_requests::create),
        )
        .route(
            "/port-requests/{id}",
            get(port_requests::get)
                .put(port_requests::update)
                .delete(port_requests::delete),
        )
        // --- Call Settings ---
        .route(
            "/call-settings",
            get(call_settings::list).post(call_settings::create),
        )
        .route(
            "/call-settings/{id}",
            get(call_settings::get)
                .put(call_settings::update)
                .delete(call_settings::delete),
        )
}
