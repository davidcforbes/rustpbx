//! CRUD handlers for `iiz.call_settings`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::numbers::{CallSetting, NewCallSetting, UpdateCallSetting};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::call_settings,
    entity: CallSetting,
    new_entity: NewCallSetting,
    update_entity: UpdateCallSetting,
);
