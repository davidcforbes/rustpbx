//! CRUD handlers for `iiz.blocked_numbers`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::contacts::{BlockedNumber, NewBlockedNumber, UpdateBlockedNumber};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::blocked_numbers,
    entity: BlockedNumber,
    new_entity: NewBlockedNumber,
    update_entity: UpdateBlockedNumber,
);
