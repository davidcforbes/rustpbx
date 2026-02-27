//! CRUD handlers for `iiz.tracking_sources`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::numbers::{NewTrackingSource, TrackingSource, UpdateTrackingSource};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::tracking_sources,
    entity: TrackingSource,
    new_entity: NewTrackingSource,
    update_entity: UpdateTrackingSource,
);
