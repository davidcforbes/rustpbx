//! CRUD handlers for `iiz.receiving_numbers`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::numbers::{NewReceivingNumber, ReceivingNumber, UpdateReceivingNumber};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::receiving_numbers,
    entity: ReceivingNumber,
    new_entity: NewReceivingNumber,
    update_entity: UpdateReceivingNumber,
);
