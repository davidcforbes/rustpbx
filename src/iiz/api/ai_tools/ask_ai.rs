//! CRUD handlers for `iiz.ask_ai_configs`.

use crate::iiz::models::ai_tools::{AskAiConfig, NewAskAiConfig, UpdateAskAiConfig};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::ask_ai_configs,
    entity: AskAiConfig,
    new_entity: NewAskAiConfig,
    update_entity: UpdateAskAiConfig,
);
