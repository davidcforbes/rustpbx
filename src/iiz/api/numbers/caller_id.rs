//! CRUD handlers for `iiz.caller_id_cnam`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::numbers::{CallerIdCnam, NewCallerIdCnam, UpdateCallerIdCnam};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::caller_id_cnam,
    entity: CallerIdCnam,
    new_entity: NewCallerIdCnam,
    update_entity: UpdateCallerIdCnam,
);
