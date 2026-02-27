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
//! - `/flows/workflows` -- workflows with nested `/workflows/{workflow_id}/nodes` and `/edges`
//! - `/flows/triggers` -- triggers with nested `/triggers/{trigger_id}/conditions` and `/actions`
//! - `/flows/lambdas` -- lambdas with nested `/lambdas/{lambda_id}/env-vars`
//! - `/flows/webhooks` -- webhooks with nested `/webhooks/{webhook_id}/subscriptions`
//! - `/flows/bulk-messages` -- bulk messages (simple CRUD)
//! - `/flows/lead-reactor` -- lead reactor configs with nested `/lead-reactor/{config_id}/actions`
//! - `/flows/smart-dialers` -- smart dialer configs (simple CRUD)
//! - `/flows/form-reactor` -- form reactor entries (simple CRUD)
//! - `/flows/chat-widgets` -- chat widgets (simple CRUD)
//! - `/flows/keyword-spotting` -- keyword spotting configs with nested keywords and numbers
//! - `/flows/reminders` -- reminders (simple CRUD)

use axum::routing::get;
use axum::Router;

use crate::iiz::api::IizState;

mod agent_scripts;
mod bulk_messages;
mod chat_widgets;
mod form_reactor;
mod geo_routers;
mod keyword_spotting;
mod lambdas;
mod lead_reactor;
mod queues;
mod reminders;
mod routing_tables;
mod schedules;
mod scoring;
mod smart_dialers;
mod smart_routers;
mod triggers;
mod voice_menus;
mod voicemails;
mod webhooks;
mod workflows;

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
        // --- Workflows + Nodes/Edges sub-resources ---
        .route(
            "/workflows",
            get(workflows::list).post(workflows::create),
        )
        .route(
            "/workflows/{id}",
            get(workflows::get)
                .put(workflows::update)
                .delete(workflows::delete),
        )
        .route(
            "/workflows/{workflow_id}/nodes",
            get(workflows::list_nodes).post(workflows::create_node),
        )
        .route(
            "/workflows/{workflow_id}/nodes/{id}",
            get(workflows::get_node)
                .put(workflows::update_node)
                .delete(workflows::delete_node),
        )
        .route(
            "/workflows/{workflow_id}/edges",
            get(workflows::list_edges).post(workflows::create_edge),
        )
        .route(
            "/workflows/{workflow_id}/edges/{id}",
            get(workflows::get_edge)
                .put(workflows::update_edge)
                .delete(workflows::delete_edge),
        )
        // --- Triggers + Conditions/Actions sub-resources ---
        .route(
            "/triggers",
            get(triggers::list).post(triggers::create),
        )
        .route(
            "/triggers/{id}",
            get(triggers::get)
                .put(triggers::update)
                .delete(triggers::delete),
        )
        .route(
            "/triggers/{trigger_id}/conditions",
            get(triggers::list_conditions).post(triggers::create_condition),
        )
        .route(
            "/triggers/{trigger_id}/conditions/{id}",
            get(triggers::get_condition)
                .put(triggers::update_condition)
                .delete(triggers::delete_condition),
        )
        .route(
            "/triggers/{trigger_id}/actions",
            get(triggers::list_actions).post(triggers::create_action),
        )
        .route(
            "/triggers/{trigger_id}/actions/{id}",
            get(triggers::get_action)
                .put(triggers::update_action)
                .delete(triggers::delete_action),
        )
        // --- Lambdas + Env Vars sub-resource ---
        .route(
            "/lambdas",
            get(lambdas::list).post(lambdas::create),
        )
        .route(
            "/lambdas/{id}",
            get(lambdas::get)
                .put(lambdas::update)
                .delete(lambdas::delete),
        )
        .route(
            "/lambdas/{lambda_id}/env-vars",
            get(lambdas::list_env_vars).post(lambdas::create_env_var),
        )
        .route(
            "/lambdas/{lambda_id}/env-vars/{id}",
            get(lambdas::get_env_var)
                .put(lambdas::update_env_var)
                .delete(lambdas::delete_env_var),
        )
        // --- Webhooks + Subscriptions sub-resource ---
        .route(
            "/webhooks",
            get(webhooks::list).post(webhooks::create),
        )
        .route(
            "/webhooks/{id}",
            get(webhooks::get)
                .put(webhooks::update)
                .delete(webhooks::delete),
        )
        .route(
            "/webhooks/{webhook_id}/subscriptions",
            get(webhooks::list_subscriptions).post(webhooks::create_subscription),
        )
        .route(
            "/webhooks/{webhook_id}/subscriptions/{id}",
            get(webhooks::get_subscription)
                .put(webhooks::update_subscription)
                .delete(webhooks::delete_subscription),
        )
        // --- Bulk Messages (simple CRUD) ---
        .route(
            "/bulk-messages",
            get(bulk_messages::list).post(bulk_messages::create),
        )
        .route(
            "/bulk-messages/{id}",
            get(bulk_messages::get)
                .put(bulk_messages::update)
                .delete(bulk_messages::delete),
        )
        // --- Lead Reactor + Actions sub-resource ---
        .route(
            "/lead-reactor",
            get(lead_reactor::list).post(lead_reactor::create),
        )
        .route(
            "/lead-reactor/{id}",
            get(lead_reactor::get)
                .put(lead_reactor::update)
                .delete(lead_reactor::delete),
        )
        .route(
            "/lead-reactor/{config_id}/actions",
            get(lead_reactor::list_actions).post(lead_reactor::create_action),
        )
        .route(
            "/lead-reactor/{config_id}/actions/{id}",
            get(lead_reactor::get_action)
                .put(lead_reactor::update_action)
                .delete(lead_reactor::delete_action),
        )
        // --- Smart Dialers (simple CRUD) ---
        .route(
            "/smart-dialers",
            get(smart_dialers::list).post(smart_dialers::create),
        )
        .route(
            "/smart-dialers/{id}",
            get(smart_dialers::get)
                .put(smart_dialers::update)
                .delete(smart_dialers::delete),
        )
        // --- Form Reactor (simple CRUD) ---
        .route(
            "/form-reactor",
            get(form_reactor::list).post(form_reactor::create),
        )
        .route(
            "/form-reactor/{id}",
            get(form_reactor::get)
                .put(form_reactor::update)
                .delete(form_reactor::delete),
        )
        // --- Chat Widgets (simple CRUD) ---
        .route(
            "/chat-widgets",
            get(chat_widgets::list).post(chat_widgets::create),
        )
        .route(
            "/chat-widgets/{id}",
            get(chat_widgets::get)
                .put(chat_widgets::update)
                .delete(chat_widgets::delete),
        )
        // --- Keyword Spotting + Keywords/Numbers sub-resources ---
        .route(
            "/keyword-spotting",
            get(keyword_spotting::list).post(keyword_spotting::create),
        )
        .route(
            "/keyword-spotting/{id}",
            get(keyword_spotting::get)
                .put(keyword_spotting::update)
                .delete(keyword_spotting::delete),
        )
        .route(
            "/keyword-spotting/{config_id}/keywords",
            get(keyword_spotting::list_keywords).post(keyword_spotting::create_keyword),
        )
        .route(
            "/keyword-spotting/{config_id}/keywords/{id}",
            get(keyword_spotting::get_keyword)
                .put(keyword_spotting::update_keyword)
                .delete(keyword_spotting::delete_keyword),
        )
        .route(
            "/keyword-spotting/{config_id}/numbers",
            get(keyword_spotting::list_numbers).post(keyword_spotting::create_number),
        )
        .route(
            "/keyword-spotting/{config_id}/numbers/{id}",
            get(keyword_spotting::get_number)
                .put(keyword_spotting::update_number)
                .delete(keyword_spotting::delete_number),
        )
        // --- Reminders (simple CRUD) ---
        .route(
            "/reminders",
            get(reminders::list).post(reminders::create),
        )
        .route(
            "/reminders/{id}",
            get(reminders::get)
                .put(reminders::update)
                .delete(reminders::delete),
        )
}
