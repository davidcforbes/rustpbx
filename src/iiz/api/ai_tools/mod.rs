//! AI Tools section router -- assembles CRUD routes for AI sub-resources.

use axum::routing::get;
use axum::Router;

use crate::iiz::api::IizState;

mod ask_ai;
mod chat_ai;
mod dialogflow;
mod knowledge_banks;
mod summaries;
mod voice_ai;

/// Build the `/ai-tools` section router.
pub fn router() -> Router<IizState> {
    Router::new()
        // --- Ask AI Configs ---
        .route(
            "/ask-ai",
            get(ask_ai::list).post(ask_ai::create),
        )
        .route(
            "/ask-ai/{id}",
            get(ask_ai::get)
                .put(ask_ai::update)
                .delete(ask_ai::delete),
        )
        // --- Summary Configs ---
        .route(
            "/summaries",
            get(summaries::list).post(summaries::create),
        )
        .route(
            "/summaries/{id}",
            get(summaries::get)
                .put(summaries::update)
                .delete(summaries::delete),
        )
        // --- Knowledge Banks + Documents sub-resource ---
        .route(
            "/knowledge-banks",
            get(knowledge_banks::list).post(knowledge_banks::create),
        )
        .route(
            "/knowledge-banks/{id}",
            get(knowledge_banks::get)
                .put(knowledge_banks::update)
                .delete(knowledge_banks::delete),
        )
        .route(
            "/knowledge-banks/{bank_id}/documents",
            get(knowledge_banks::list_documents).post(knowledge_banks::create_document),
        )
        .route(
            "/knowledge-banks/{bank_id}/documents/{id}",
            get(knowledge_banks::get_document)
                .put(knowledge_banks::update_document)
                .delete(knowledge_banks::delete_document),
        )
        // --- Voice AI Agents ---
        .route(
            "/voice-ai",
            get(voice_ai::list).post(voice_ai::create),
        )
        .route(
            "/voice-ai/{id}",
            get(voice_ai::get)
                .put(voice_ai::update)
                .delete(voice_ai::delete),
        )
        // --- Chat AI Agents ---
        .route(
            "/chat-ai/agents",
            get(chat_ai::list_agents).post(chat_ai::create_agent),
        )
        .route(
            "/chat-ai/agents/{id}",
            get(chat_ai::get_agent)
                .put(chat_ai::update_agent)
                .delete(chat_ai::delete_agent),
        )
        // --- Chat AI Configs ---
        .route(
            "/chat-ai/configs",
            get(chat_ai::list_configs).post(chat_ai::create_config),
        )
        .route(
            "/chat-ai/configs/{id}",
            get(chat_ai::get_config)
                .put(chat_ai::update_config)
                .delete(chat_ai::delete_config),
        )
        // --- Dialogflow Configs ---
        .route(
            "/dialogflow",
            get(dialogflow::list).post(dialogflow::create),
        )
        .route(
            "/dialogflow/{id}",
            get(dialogflow::get)
                .put(dialogflow::update)
                .delete(dialogflow::delete),
        )
}
