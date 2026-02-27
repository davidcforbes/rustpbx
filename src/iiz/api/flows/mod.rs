//! Flow section router -- assembles CRUD routes for all flow sub-resources.
//!
//! Sub-resources:
//! - `/flows/queues` -- call queues with nested `/queues/{queue_id}/agents`
//! - `/flows/routing-tables` -- routing tables with nested `/routing-tables/{table_id}/routes`
//! - `/flows/schedules` -- business-hours schedules with nested `/schedules/{schedule_id}/holidays`
//! - `/flows/voice-menus` -- IVR voice menus with nested `/voice-menus/{menu_id}/options`
//! - `/flows/voicemails` -- voicemail boxes with nested `/voicemails/{mailbox_id}/messages`
//! - `/flows/geo-routers` -- geographic routers with nested `/geo-routers/{router_id}/rules`
//! - `/flows/smart-routers` -- smart routers with nested `/smart-routers/{router_id}/rules`
//! - `/flows/agent-scripts` -- agent scripts (simple CRUD)
//! - `/flows/scoring` -- scoring configs (simple CRUD)

use axum::routing::get;
use axum::Router;

use crate::iiz::api::IizState;

mod agent_scripts;
mod geo_routers;
mod queues;
mod routing_tables;
mod schedules;
mod scoring;
mod smart_routers;
mod voice_menus;
mod voicemails;

/// Build the `/flows` section router.
///
/// All routes require authentication (via `AuthContext` extractor) and use
/// RLS-scoped connections for tenant isolation.
pub fn router() -> Router<IizState> {
    Router::new()
        // --- Queues + Agents sub-resource ---
        .route(
            "/queues",
            get(queues::list).post(queues::create),
        )
        .route(
            "/queues/{id}",
            get(queues::get)
                .put(queues::update)
                .delete(queues::delete),
        )
        .route(
            "/queues/{queue_id}/agents",
            get(queues::list_agents).post(queues::create_agent),
        )
        .route(
            "/queues/{queue_id}/agents/{id}",
            get(queues::get_agent)
                .put(queues::update_agent)
                .delete(queues::delete_agent),
        )
        // --- Routing Tables + Routes sub-resource ---
        .route(
            "/routing-tables",
            get(routing_tables::list).post(routing_tables::create),
        )
        .route(
            "/routing-tables/{id}",
            get(routing_tables::get)
                .put(routing_tables::update)
                .delete(routing_tables::delete),
        )
        .route(
            "/routing-tables/{table_id}/routes",
            get(routing_tables::list_routes).post(routing_tables::create_route),
        )
        .route(
            "/routing-tables/{table_id}/routes/{id}",
            get(routing_tables::get_route)
                .put(routing_tables::update_route)
                .delete(routing_tables::delete_route),
        )
        // --- Schedules + Holidays sub-resource ---
        .route(
            "/schedules",
            get(schedules::list).post(schedules::create),
        )
        .route(
            "/schedules/{id}",
            get(schedules::get)
                .put(schedules::update)
                .delete(schedules::delete),
        )
        .route(
            "/schedules/{schedule_id}/holidays",
            get(schedules::list_holidays).post(schedules::create_holiday),
        )
        .route(
            "/schedules/{schedule_id}/holidays/{id}",
            get(schedules::get_holiday)
                .put(schedules::update_holiday)
                .delete(schedules::delete_holiday),
        )
        // --- Voice Menus + Options sub-resource ---
        .route(
            "/voice-menus",
            get(voice_menus::list).post(voice_menus::create),
        )
        .route(
            "/voice-menus/{id}",
            get(voice_menus::get)
                .put(voice_menus::update)
                .delete(voice_menus::delete),
        )
        .route(
            "/voice-menus/{menu_id}/options",
            get(voice_menus::list_options).post(voice_menus::create_option),
        )
        .route(
            "/voice-menus/{menu_id}/options/{id}",
            get(voice_menus::get_option)
                .put(voice_menus::update_option)
                .delete(voice_menus::delete_option),
        )
        // --- Voicemail Boxes + Messages sub-resource ---
        .route(
            "/voicemails",
            get(voicemails::list).post(voicemails::create),
        )
        .route(
            "/voicemails/{id}",
            get(voicemails::get)
                .put(voicemails::update)
                .delete(voicemails::delete),
        )
        .route(
            "/voicemails/{mailbox_id}/messages",
            get(voicemails::list_messages).post(voicemails::create_message),
        )
        .route(
            "/voicemails/{mailbox_id}/messages/{id}",
            get(voicemails::get_message)
                .put(voicemails::update_message)
                .delete(voicemails::delete_message),
        )
        // --- Geo Routers + Rules sub-resource ---
        .route(
            "/geo-routers",
            get(geo_routers::list).post(geo_routers::create),
        )
        .route(
            "/geo-routers/{id}",
            get(geo_routers::get)
                .put(geo_routers::update)
                .delete(geo_routers::delete),
        )
        .route(
            "/geo-routers/{router_id}/rules",
            get(geo_routers::list_rules).post(geo_routers::create_rule),
        )
        .route(
            "/geo-routers/{router_id}/rules/{id}",
            get(geo_routers::get_rule)
                .put(geo_routers::update_rule)
                .delete(geo_routers::delete_rule),
        )
        // --- Smart Routers + Rules sub-resource ---
        .route(
            "/smart-routers",
            get(smart_routers::list).post(smart_routers::create),
        )
        .route(
            "/smart-routers/{id}",
            get(smart_routers::get)
                .put(smart_routers::update)
                .delete(smart_routers::delete),
        )
        .route(
            "/smart-routers/{router_id}/rules",
            get(smart_routers::list_rules).post(smart_routers::create_rule),
        )
        .route(
            "/smart-routers/{router_id}/rules/{id}",
            get(smart_routers::get_rule)
                .put(smart_routers::update_rule)
                .delete(smart_routers::delete_rule),
        )
        // --- Agent Scripts (simple CRUD) ---
        .route(
            "/agent-scripts",
            get(agent_scripts::list).post(agent_scripts::create),
        )
        .route(
            "/agent-scripts/{id}",
            get(agent_scripts::get)
                .put(agent_scripts::update)
                .delete(agent_scripts::delete),
        )
        // --- Scoring Configs (simple CRUD) ---
        .route(
            "/scoring",
            get(scoring::list).post(scoring::create),
        )
        .route(
            "/scoring/{id}",
            get(scoring::get)
                .put(scoring::update)
                .delete(scoring::delete),
        )
}
