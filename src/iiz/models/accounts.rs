//! Diesel model structs for the `accounts`, `users`, and `account_variables` tables.
//!
//! Each table has three structs:
//! - Read model (Queryable/Selectable) for SELECT queries
//! - Insert model (Insertable) for INSERT queries
//! - Update model (AsChangeset) for partial UPDATE queries

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::iiz::models::enums::{AccountStatus, AccountType, UserRole};
use crate::iiz::schema::iiz::{account_variables, accounts, users};

// ---------------------------------------------------------------------------
// accounts
// ---------------------------------------------------------------------------

/// Read model for the `iiz.accounts` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = accounts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub account_type: AccountType,
    pub parent_account_id: Option<Uuid>,
    pub slug: String,
    pub timezone: String,
    pub status: AccountStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new account.
/// `id`, `created_at`, and `updated_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = accounts)]
pub struct NewAccount {
    pub name: String,
    pub account_type: AccountType,
    pub parent_account_id: Option<Uuid>,
    pub slug: String,
    pub timezone: String,
    pub status: AccountStatus,
}

/// Update model for partial account updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = accounts)]
pub struct UpdateAccount {
    pub name: Option<String>,
    pub account_type: Option<AccountType>,
    pub parent_account_id: Option<Option<Uuid>>,
    pub slug: Option<String>,
    pub timezone: Option<String>,
    pub status: Option<AccountStatus>,
}

// ---------------------------------------------------------------------------
// users
// ---------------------------------------------------------------------------

/// Read model for the `iiz.users` table.
///
/// Sensitive fields (`password_hash`, `reset_token`, `reset_token_expires`)
/// are skipped during serialization to prevent accidental leakage.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub account_id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub display_name: Option<String>,
    pub initials: Option<String>,
    pub avatar_color: Option<String>,
    pub role: UserRole,
    pub phone: Option<String>,
    pub is_active: bool,
    #[serde(skip_serializing)]
    pub reset_token: Option<String>,
    #[serde(skip_serializing)]
    pub reset_token_expires: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new user.
/// `id`, `created_at`, and `updated_at` are set by database defaults.
/// `account_id` is required because users belong to a specific account.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub account_id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub initials: Option<String>,
    pub avatar_color: Option<String>,
    pub role: UserRole,
    pub phone: Option<String>,
    pub is_active: bool,
}

/// Update model for partial user updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// Nullable columns use `Option<Option<T>>` so they can be explicitly set to NULL.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub display_name: Option<Option<String>>,
    pub initials: Option<Option<String>>,
    pub avatar_color: Option<Option<String>>,
    pub role: Option<UserRole>,
    pub phone: Option<Option<String>>,
    pub is_active: Option<bool>,
    pub reset_token: Option<Option<String>>,
    pub reset_token_expires: Option<Option<DateTime<Utc>>>,
    pub last_login_at: Option<Option<DateTime<Utc>>>,
    pub last_login_ip: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// account_variables
// ---------------------------------------------------------------------------

/// Read model for the `iiz.account_variables` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = account_variables)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AccountVariable {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub value: Option<String>,
    pub description: Option<String>,
    pub is_secret: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new account variable.
/// `id`, `created_at`, and `updated_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = account_variables)]
pub struct NewAccountVariable {
    pub account_id: Uuid,
    pub name: String,
    pub value: Option<String>,
    pub description: Option<String>,
    pub is_secret: bool,
}

/// Update model for partial account variable updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = account_variables)]
pub struct UpdateAccountVariable {
    pub name: Option<String>,
    pub value: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub is_secret: Option<bool>,
}
