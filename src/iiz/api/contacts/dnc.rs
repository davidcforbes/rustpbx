//! CRUD handlers for `iiz.dnc_entries` (Do-Not-Call).
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::contacts::{DncEntry, NewDncEntry, UpdateDncEntry};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::dnc_entries,
    entity: DncEntry,
    new_entity: NewDncEntry,
    update_entity: UpdateDncEntry,
);
