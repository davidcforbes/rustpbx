//! CRUD handlers for `iiz.port_requests`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::numbers::{NewPortRequest, PortRequest, UpdatePortRequest};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::port_requests,
    entity: PortRequest,
    new_entity: NewPortRequest,
    update_entity: UpdatePortRequest,
);
