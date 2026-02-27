//! CRUD handlers for `iiz.target_numbers`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::numbers::{NewTargetNumber, TargetNumber, UpdateTargetNumber};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::target_numbers,
    entity: TargetNumber,
    new_entity: NewTargetNumber,
    update_entity: UpdateTargetNumber,
);
