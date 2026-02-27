//! CRUD handlers for `iiz.tags`.
//!
//! Uses the `crud_handlers!` macro to generate standard list/get/create/update/delete
//! endpoints. RLS provides account_id scoping and soft-delete filtering.

use crate::iiz::models::tags::{NewTag, Tag, UpdateTag};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::tags,
    entity: Tag,
    new_entity: NewTag,
    update_entity: UpdateTag,
);
