//! CRUD handlers for `iiz.form_reactor_entries`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::engagement::{FormReactorEntry, NewFormReactorEntry, UpdateFormReactorEntry};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::form_reactor_entries,
    entity: FormReactorEntry,
    new_entity: NewFormReactorEntry,
    update_entity: UpdateFormReactorEntry,
);
