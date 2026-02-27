//! CRUD handlers for `iiz.voice_ai_agents`.

use crate::iiz::models::ai_tools::{NewVoiceAiAgent, UpdateVoiceAiAgent, VoiceAiAgent};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::voice_ai_agents,
    entity: VoiceAiAgent,
    new_entity: NewVoiceAiAgent,
    update_entity: UpdateVoiceAiAgent,
);
