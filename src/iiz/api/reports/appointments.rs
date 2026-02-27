//! CRUD handlers for appointments.

use crate::iiz::models::reports::{Appointment, NewAppointment, UpdateAppointment};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::appointments,
    entity: Appointment,
    new_entity: NewAppointment,
    update_entity: UpdateAppointment,
);
