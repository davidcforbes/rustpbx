//! CRUD handlers for frequency_limits.

use crate::iiz::models::ephemeral::{FrequencyLimit, NewFrequencyLimit, UpdateFrequencyLimit};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::frequency_limits,
    entity: FrequencyLimit,
    new_entity: NewFrequencyLimit,
    update_entity: UpdateFrequencyLimit,
);
