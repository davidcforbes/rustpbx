//! CRUD handlers for active_calls (UNLOGGED table for real-time state).

use crate::iiz::models::activities::{ActiveCall, NewActiveCall, UpdateActiveCall};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::active_calls,
    entity: ActiveCall,
    new_entity: NewActiveCall,
    update_entity: UpdateActiveCall,
);
