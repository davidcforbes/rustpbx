//! CRUD handlers for `iiz.text_numbers`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::numbers::{NewTextNumber, TextNumber, UpdateTextNumber};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::text_numbers,
    entity: TextNumber,
    new_entity: NewTextNumber,
    update_entity: UpdateTextNumber,
);
