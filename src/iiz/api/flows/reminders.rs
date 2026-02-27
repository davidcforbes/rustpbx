//! CRUD handlers for `iiz.reminders`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::engagement::{NewReminder, Reminder, UpdateReminder};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::reminders,
    entity: Reminder,
    new_entity: NewReminder,
    update_entity: UpdateReminder,
);
