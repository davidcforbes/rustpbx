//! CRUD handlers for `iiz.toll_free_registrations`.

use crate::iiz::models::trust_center::{TollFreeRegistration, NewTollFreeRegistration, UpdateTollFreeRegistration};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::toll_free_registrations,
    entity: TollFreeRegistration,
    new_entity: NewTollFreeRegistration,
    update_entity: UpdateTollFreeRegistration,
);
