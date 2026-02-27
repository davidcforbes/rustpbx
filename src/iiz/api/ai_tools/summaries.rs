//! CRUD handlers for `iiz.summary_configs`.

use crate::iiz::models::ai_tools::{NewSummaryConfig, SummaryConfig, UpdateSummaryConfig};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::summary_configs,
    entity: SummaryConfig,
    new_entity: NewSummaryConfig,
    update_entity: UpdateSummaryConfig,
);
