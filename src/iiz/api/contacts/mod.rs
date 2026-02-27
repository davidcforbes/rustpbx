//! Contact section router — assembles CRUD routes for all contact sub-resources.
//!
//! Sub-resources:
//! - `/contacts/lists` — contact lists with nested `/lists/{list_id}/members`
//! - `/contacts/blocked` — blocked phone numbers
//! - `/contacts/dnc` — do-not-call entries
//! - `/contacts/dnt` — do-not-text entries

use axum::routing::get;
use axum::Router;

use crate::iiz::api::IizState;

mod blocked_numbers;
mod contact_lists;
mod dnc;
mod dnt;

/// Build the `/contacts` section router.
///
/// All routes require authentication (via `AuthContext` extractor) and use
/// RLS-scoped connections for tenant isolation.
pub fn router() -> Router<IizState> {
    Router::new()
        // --- Contact Lists + Members sub-resource ---
        .route(
            "/lists",
            get(contact_lists::list).post(contact_lists::create),
        )
        .route(
            "/lists/{id}",
            get(contact_lists::get)
                .put(contact_lists::update)
                .delete(contact_lists::delete),
        )
        .route(
            "/lists/{list_id}/members",
            get(contact_lists::list_members).post(contact_lists::create_member),
        )
        .route(
            "/lists/{list_id}/members/{id}",
            get(contact_lists::get_member)
                .put(contact_lists::update_member)
                .delete(contact_lists::delete_member),
        )
        // --- Blocked Numbers ---
        .route(
            "/blocked",
            get(blocked_numbers::list).post(blocked_numbers::create),
        )
        .route(
            "/blocked/{id}",
            get(blocked_numbers::get)
                .put(blocked_numbers::update)
                .delete(blocked_numbers::delete),
        )
        // --- Do-Not-Call ---
        .route("/dnc", get(dnc::list).post(dnc::create))
        .route(
            "/dnc/{id}",
            get(dnc::get).put(dnc::update).delete(dnc::delete),
        )
        // --- Do-Not-Text ---
        .route("/dnt", get(dnt::list).post(dnt::create))
        .route(
            "/dnt/{id}",
            get(dnt::get).put(dnt::update).delete(dnt::delete),
        )
}
