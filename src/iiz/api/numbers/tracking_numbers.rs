//! CRUD handlers for `iiz.tracking_numbers`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::numbers::{NewTrackingNumber, TrackingNumber, UpdateTrackingNumber};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::tracking_numbers,
    entity: TrackingNumber,
    new_entity: NewTrackingNumber,
    update_entity: UpdateTrackingNumber,
);
