//! Diesel model structs for the iiz schema.
//!
//! Each struct derives Queryable, Insertable, and Serde traits for use with
//! the Diesel ORM and API serialization.

pub mod accounts;
pub mod contacts;
pub mod enums;
pub mod numbers;
pub mod tags;

// Re-export enum types for convenience
pub use enums::*;

// Re-export model structs for convenience
pub use accounts::{
    Account, AccountVariable, NewAccount, NewAccountVariable, NewUser, UpdateAccount,
    UpdateAccountVariable, UpdateUser, User,
};
pub use contacts::{
    BlockedNumber, ContactList, ContactListMember, DncEntry, DntEntry, NewBlockedNumber,
    NewContactList, NewContactListMember, NewDncEntry, NewDntEntry, UpdateBlockedNumber,
    UpdateContactList, UpdateContactListMember, UpdateDncEntry, UpdateDntEntry,
};
pub use numbers::{
    CallSetting, CallerIdCnam, NewCallSetting, NewCallerIdCnam, NewNumberPool,
    NewNumberPoolMember, NewPortRequest, NewReceivingNumber, NewTargetNumber, NewTextNumber,
    NewTrackingNumber, NewTrackingSource, NumberPool, NumberPoolMember, PortRequest,
    ReceivingNumber, TargetNumber, TextNumber, TrackingNumber, TrackingSource, UpdateCallSetting,
    UpdateCallerIdCnam, UpdateNumberPool, UpdateNumberPoolMember, UpdatePortRequest,
    UpdateReceivingNumber, UpdateTargetNumber, UpdateTextNumber, UpdateTrackingNumber,
    UpdateTrackingSource,
};
pub use tags::{NewTag, Tag, UpdateTag};

// Common type aliases used across models
pub use chrono::{DateTime, Utc};
pub use uuid::Uuid;
