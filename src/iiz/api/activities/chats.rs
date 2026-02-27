//! CRUD handlers for `iiz.chat_records`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::communication::{ChatRecord, NewChatRecord, UpdateChatRecord};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::chat_records,
    entity: ChatRecord,
    new_entity: NewChatRecord,
    update_entity: UpdateChatRecord,
);
