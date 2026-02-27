//! CRUD handlers for locations (UNLOGGED SIP registration table).

use crate::iiz::models::ephemeral::{Location, NewLocation, UpdateLocation};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::locations,
    entity: Location,
    new_entity: NewLocation,
    update_entity: UpdateLocation,
);
