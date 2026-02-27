//! CRUD handlers for `iiz.scoring_configs`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::flows::{NewScoringConfig, ScoringConfig, UpdateScoringConfig};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::scoring_configs,
    entity: ScoringConfig,
    new_entity: NewScoringConfig,
    update_entity: UpdateScoringConfig,
);
