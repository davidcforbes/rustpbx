//! Reports section router -- custom reports, notifications, and appointments.

use axum::routing::get;
use axum::Router;

use crate::iiz::api::IizState;

mod appointments;
mod custom_reports;
mod notifications;

/// Build the `/reports` section router.
pub fn router() -> Router<IizState> {
    Router::new()
        // --- Custom Reports ---
        .route(
            "/custom-reports",
            get(custom_reports::list).post(custom_reports::create),
        )
        .route(
            "/custom-reports/{id}",
            get(custom_reports::get)
                .put(custom_reports::update)
                .delete(custom_reports::delete),
        )
        // --- Notification Rules ---
        .route(
            "/notification-rules",
            get(notifications::list_rules).post(notifications::create_rule),
        )
        .route(
            "/notification-rules/{id}",
            get(notifications::get_rule)
                .put(notifications::update_rule)
                .delete(notifications::delete_rule),
        )
        // --- Notifications ---
        .route(
            "/notifications",
            get(notifications::list_notifications).post(notifications::create_notification),
        )
        .route(
            "/notifications/{id}",
            get(notifications::get_notification)
                .put(notifications::update_notification)
                .delete(notifications::delete_notification),
        )
        // --- Appointments ---
        .route(
            "/appointments",
            get(appointments::list).post(appointments::create),
        )
        .route(
            "/appointments/{id}",
            get(appointments::get)
                .put(appointments::update)
                .delete(appointments::delete),
        )
}
