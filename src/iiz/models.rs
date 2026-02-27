//! Diesel model structs for the iiz schema.
//!
//! Each struct derives Queryable, Insertable, and Serde traits for use with
//! the Diesel ORM and API serialization.
//!
//! Models will be added as features are implemented against the iiz tables.
//! For now this module re-exports common types used across models.

pub use chrono::{DateTime, Utc};
pub use uuid::Uuid;
