//! CRUD handlers for `iiz.dnt_entries` (Do-Not-Text).
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::contacts::{DntEntry, NewDntEntry, UpdateDntEntry};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::dnt_entries,
    entity: DntEntry,
    new_entity: NewDntEntry,
    update_entity: UpdateDntEntry,
);
