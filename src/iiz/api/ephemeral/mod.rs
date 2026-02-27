//! Ephemeral section router -- real-time tables (UNLOGGED) and rate limits.

use axum::routing::get;
use axum::Router;

use crate::iiz::api::IizState;

mod active_calls;
mod frequency_limits;
mod locations;
mod presence;

/// Build the `/ephemeral` section router.
pub fn router() -> Router<IizState> {
    Router::new()
        // --- Active Calls ---
        .route(
            "/active-calls",
            get(active_calls::list).post(active_calls::create),
        )
        .route(
            "/active-calls/{id}",
            get(active_calls::get)
                .put(active_calls::update)
                .delete(active_calls::delete),
        )
        // --- Presence (TEXT PK: identity) ---
        .route(
            "/presence",
            get(presence::list).post(presence::create),
        )
        .route(
            "/presence/{identity}",
            get(presence::get)
                .put(presence::update)
                .delete(presence::delete),
        )
        // --- Locations ---
        .route(
            "/locations",
            get(locations::list).post(locations::create),
        )
        .route(
            "/locations/{id}",
            get(locations::get)
                .put(locations::update)
                .delete(locations::delete),
        )
        // --- Frequency Limits ---
        .route(
            "/frequency-limits",
            get(frequency_limits::list).post(frequency_limits::create),
        )
        .route(
            "/frequency-limits/{id}",
            get(frequency_limits::get)
                .put(frequency_limits::update)
                .delete(frequency_limits::delete),
        )
}
