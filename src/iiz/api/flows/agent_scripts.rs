//! CRUD handlers for `iiz.agent_scripts`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::flows::{AgentScript, NewAgentScript, UpdateAgentScript};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::agent_scripts,
    entity: AgentScript,
    new_entity: NewAgentScript,
    update_entity: UpdateAgentScript,
);
