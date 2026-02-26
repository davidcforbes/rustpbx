//! SeaORM entities generated from the iiz PostgreSQL schema.
//!
//! Regenerate after schema changes:
//! ```bash
//! sea-orm-cli generate entity \
//!     --database-url "postgres://user:pass@localhost/rustpbx" \
//!     --database-schema iiz \
//!     --output-dir src/iiz/entities/generated \
//!     --with-serde both \
//!     --date-time-crate chrono
//! ```

pub mod extensions;

// Generated entities will be re-exported here after first generation.
// Uncomment after running sea-orm-cli:
// pub mod generated;
// pub use generated::*;
