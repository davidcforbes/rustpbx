//! CRUD handlers for custom_reports.

use crate::iiz::models::reports::{CustomReport, NewCustomReport, UpdateCustomReport};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::custom_reports,
    entity: CustomReport,
    new_entity: NewCustomReport,
    update_entity: UpdateCustomReport,
);
