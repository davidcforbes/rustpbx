//! CRUD handlers for `iiz.chat_ai_agents` and `iiz.chat_ai_configs`.

mod agents {
    use crate::iiz::models::ai_tools::{ChatAiAgent, NewChatAiAgent, UpdateChatAiAgent};

    crate::crud_handlers!(
        table: crate::iiz::schema::iiz::chat_ai_agents,
        entity: ChatAiAgent,
        new_entity: NewChatAiAgent,
        update_entity: UpdateChatAiAgent,
    );
}

mod configs {
    use crate::iiz::models::ai_tools::{ChatAiConfig, NewChatAiConfig, UpdateChatAiConfig};

    crate::crud_handlers!(
        table: crate::iiz::schema::iiz::chat_ai_configs,
        entity: ChatAiConfig,
        new_entity: NewChatAiConfig,
        update_entity: UpdateChatAiConfig,
    );
}

// Re-export with prefixed names for the router
pub use agents::create as create_agent;
pub use agents::delete as delete_agent;
pub use agents::get as get_agent;
pub use agents::list as list_agents;
pub use agents::update as update_agent;

pub use configs::create as create_config;
pub use configs::delete as delete_config;
pub use configs::get as get_config;
pub use configs::list as list_configs;
pub use configs::update as update_config;
