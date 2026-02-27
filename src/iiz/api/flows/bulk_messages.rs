//! CRUD handlers for `iiz.bulk_messages`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::engagement::{BulkMessage, NewBulkMessage, UpdateBulkMessage};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::bulk_messages,
    entity: BulkMessage,
    new_entity: NewBulkMessage,
    update_entity: UpdateBulkMessage,
);
