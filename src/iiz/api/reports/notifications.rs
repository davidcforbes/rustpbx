//! CRUD handlers for notification_rules and notifications.

mod rules {
    use crate::iiz::models::reports::{NotificationRule, NewNotificationRule, UpdateNotificationRule};

    crate::crud_handlers!(
        table: crate::iiz::schema::iiz::notification_rules,
        entity: NotificationRule,
        new_entity: NewNotificationRule,
        update_entity: UpdateNotificationRule,
    );
}

mod notifs {
    use crate::iiz::models::reports::{Notification, NewNotification, UpdateNotification};

    crate::crud_handlers!(
        table: crate::iiz::schema::iiz::notifications,
        entity: Notification,
        new_entity: NewNotification,
        update_entity: UpdateNotification,
    );
}

// Re-export with prefixes to avoid name collisions
pub use rules::list as list_rules;
pub use rules::get as get_rule;
pub use rules::create as create_rule;
pub use rules::update as update_rule;
pub use rules::delete as delete_rule;

pub use notifs::list as list_notifications;
pub use notifs::get as get_notification;
pub use notifs::create as create_notification;
pub use notifs::update as update_notification;
pub use notifs::delete as delete_notification;
