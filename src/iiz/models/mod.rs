//! Diesel model structs for the iiz schema.
//!
//! Each struct derives Queryable, Insertable, and Serde traits for use with
//! the Diesel ORM and API serialization.

pub mod enums;

// Re-export enum types for convenience
pub use enums::*;

// Common type aliases used across models
pub use chrono::{DateTime, Utc};
pub use uuid::Uuid;
