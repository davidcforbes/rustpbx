//! Trust Center section router -- compliance, registrations, and business info.

use axum::routing::get;
use axum::Router;

use crate::iiz::api::IizState;

mod a2p_campaigns;
mod addresses;
mod business_info;
mod compliance;
mod toll_free;
mod voice_registrations;

/// Build the `/trust-center` section router.
pub fn router() -> Router<IizState> {
    Router::new()
        // --- Business Info + Authorized Contacts ---
        .route(
            "/business-info",
            get(business_info::list).post(business_info::create),
        )
        .route(
            "/business-info/{id}",
            get(business_info::get)
                .put(business_info::update)
                .delete(business_info::delete),
        )
        .route(
            "/business-info/{info_id}/contacts",
            get(business_info::list_contacts).post(business_info::create_contact),
        )
        .route(
            "/business-info/{info_id}/contacts/{id}",
            get(business_info::get_contact)
                .put(business_info::update_contact)
                .delete(business_info::delete_contact),
        )
        // --- Compliance Requirements ---
        .route(
            "/requirements",
            get(compliance::list_requirements).post(compliance::create_requirement),
        )
        .route(
            "/requirements/{id}",
            get(compliance::get_requirement)
                .put(compliance::update_requirement)
                .delete(compliance::delete_requirement),
        )
        // --- Compliance Applications ---
        .route(
            "/applications",
            get(compliance::list_applications).post(compliance::create_application),
        )
        .route(
            "/applications/{id}",
            get(compliance::get_application)
                .put(compliance::update_application)
                .delete(compliance::delete_application),
        )
        // --- Compliance Addresses ---
        .route(
            "/addresses",
            get(addresses::list).post(addresses::create),
        )
        .route(
            "/addresses/{id}",
            get(addresses::get)
                .put(addresses::update)
                .delete(addresses::delete),
        )
        // --- Voice Registrations + History ---
        .route(
            "/voice-registrations",
            get(voice_registrations::list).post(voice_registrations::create),
        )
        .route(
            "/voice-registrations/{id}",
            get(voice_registrations::get)
                .put(voice_registrations::update)
                .delete(voice_registrations::delete),
        )
        .route(
            "/voice-registrations/{reg_id}/history",
            get(voice_registrations::list_history).post(voice_registrations::create_history),
        )
        // --- A2P Campaigns ---
        .route(
            "/a2p-campaigns",
            get(a2p_campaigns::list).post(a2p_campaigns::create),
        )
        .route(
            "/a2p-campaigns/{id}",
            get(a2p_campaigns::get)
                .put(a2p_campaigns::update)
                .delete(a2p_campaigns::delete),
        )
        // --- Toll-Free Registrations ---
        .route(
            "/toll-free",
            get(toll_free::list).post(toll_free::create),
        )
        .route(
            "/toll-free/{id}",
            get(toll_free::get)
                .put(toll_free::update)
                .delete(toll_free::delete),
        )
}
