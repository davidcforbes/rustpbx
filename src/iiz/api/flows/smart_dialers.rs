//! CRUD handlers for `iiz.smart_dialer_configs`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::engagement::{NewSmartDialerConfig, SmartDialerConfig, UpdateSmartDialerConfig};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::smart_dialer_configs,
    entity: SmartDialerConfig,
    new_entity: NewSmartDialerConfig,
    update_entity: UpdateSmartDialerConfig,
);
