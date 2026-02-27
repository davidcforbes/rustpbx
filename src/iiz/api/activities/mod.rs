//! Activities section router -- call records, texts, chats, forms, fax, video, exports.
//!
//! Call records are read-only from the API (created by the call processing pipeline).
//! Sub-resources under `/calls/{call_id}/...` provide access to flow events,
//! transcription segments, AI summaries, annotations, tags, keyword hits, and
//! visitor sessions.
//!
//! Communication records (texts, chats, forms, fax, video) and exports have
//! standard CRUD or read+insert endpoints.

use axum::routing::get;
use axum::Router;

use crate::iiz::api::IizState;

mod calls;
mod chats;
mod exports;
mod fax;
mod forms;
mod texts;
mod video;

/// Build the `/activities` section router.
///
/// All routes require authentication (via `AuthContext` extractor) and use
/// RLS-scoped connections for tenant isolation.
pub fn router() -> Router<IizState> {
    Router::new()
        // --- Calls (read-only list/get) + sub-resources ---
        .route("/calls", get(calls::list))
        .route("/calls/{id}", get(calls::get))
        .route(
            "/calls/{call_id}/flow",
            get(calls::list_flow_events),
        )
        .route(
            "/calls/{call_id}/transcription",
            get(calls::list_transcription_segments),
        )
        .route(
            "/calls/{call_id}/summaries",
            get(calls::list_summaries),
        )
        .route(
            "/calls/{call_id}/annotations",
            get(calls::get_annotation).put(calls::upsert_annotation),
        )
        .route(
            "/calls/{call_id}/tags",
            get(calls::list_tags).post(calls::create_tag),
        )
        .route(
            "/calls/{call_id}/tags/{id}",
            axum::routing::delete(calls::delete_tag),
        )
        .route(
            "/calls/{call_id}/keyword-hits",
            get(calls::list_keyword_hits),
        )
        .route(
            "/calls/{call_id}/visitor",
            get(calls::get_visitor_session),
        )
        // --- Texts (list, get, create, soft-delete — no update) ---
        .route("/texts", get(texts::list).post(texts::create))
        .route(
            "/texts/{id}",
            get(texts::get).delete(texts::delete),
        )
        // --- Chats (full CRUD) ---
        .route("/chats", get(chats::list).post(chats::create))
        .route(
            "/chats/{id}",
            get(chats::get)
                .put(chats::update)
                .delete(chats::delete),
        )
        // --- Forms (list, get, create, soft-delete — no update) ---
        .route("/forms", get(forms::list).post(forms::create))
        .route(
            "/forms/{id}",
            get(forms::get).delete(forms::delete),
        )
        // --- Fax (list, get, create, soft-delete — no update) ---
        .route("/fax", get(fax::list).post(fax::create))
        .route(
            "/fax/{id}",
            get(fax::get).delete(fax::delete),
        )
        // --- Video (list, get, create, soft-delete — no update) ---
        .route("/video", get(video::list).post(video::create))
        .route(
            "/video/{id}",
            get(video::get).delete(video::delete),
        )
        // --- Exports (full CRUD) ---
        .route("/exports", get(exports::list).post(exports::create))
        .route(
            "/exports/{id}",
            get(exports::get)
                .put(exports::update)
                .delete(exports::delete),
        )
}
