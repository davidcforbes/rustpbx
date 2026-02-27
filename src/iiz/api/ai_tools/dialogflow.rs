//! CRUD handlers for `iiz.dialogflow_configs`.

use crate::iiz::models::ai_tools::{DialogflowConfig, NewDialogflowConfig, UpdateDialogflowConfig};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::dialogflow_configs,
    entity: DialogflowConfig,
    new_entity: NewDialogflowConfig,
    update_entity: UpdateDialogflowConfig,
);
