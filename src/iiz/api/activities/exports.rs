//! CRUD handlers for `iiz.export_records`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::communication::{ExportRecord, NewExportRecord, UpdateExportRecord};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::export_records,
    entity: ExportRecord,
    new_entity: NewExportRecord,
    update_entity: UpdateExportRecord,
);
