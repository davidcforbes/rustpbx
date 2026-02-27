//! CRUD handlers for `iiz.chat_widgets`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::engagement::{ChatWidget, NewChatWidget, UpdateChatWidget};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::chat_widgets,
    entity: ChatWidget,
    new_entity: NewChatWidget,
    update_entity: UpdateChatWidget,
);
