//! Voicemail addon stub.
//!
//! The full voicemail implementation lives in `src/voicemail/` (the core
//! `VoicemailService`).  This addon module provides the `Addon` trait
//! implementation so the voicemail subsystem can be registered with the
//! addon registry and participate in lifecycle management (initialize,
//! router, sidebar items, etc.).
//!
//! When the `addon-voicemail` feature is enabled, `VoicemailAddon` is
//! registered automatically by the `AddonRegistry`.

use crate::addons::{Addon, AddonCategory, SidebarItem};
use crate::app::AppState;
use async_trait::async_trait;
use axum::Router;

pub struct VoicemailAddon;

impl VoicemailAddon {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Addon for VoicemailAddon {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn id(&self) -> &'static str {
        "voicemail"
    }

    fn name(&self) -> &'static str {
        "Voicemail"
    }

    fn description(&self) -> &'static str {
        "Visual voicemail with greeting management, MWI notifications, and email forwarding."
    }

    fn category(&self) -> AddonCategory {
        AddonCategory::Community
    }

    fn cost(&self) -> &'static str {
        "Free"
    }

    async fn initialize(&self, _state: AppState) -> anyhow::Result<()> {
        tracing::info!("VoicemailAddon initialized (stub)");
        Ok(())
    }

    fn router(&self, _state: AppState) -> Option<Router> {
        // TODO: wire up voicemail REST API routes here
        None
    }

    fn sidebar_items(&self, _state: AppState) -> Vec<SidebarItem> {
        vec![SidebarItem {
            name: "Voicemail".to_string(),
            icon: r#"<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="M21.75 9v.906a2.25 2.25 0 01-1.183 1.981l-6.478 3.488M2.25 9v.906a2.25 2.25 0 001.183 1.981l6.478 3.488m8.839 2.51l-4.66-2.51m0 0l-1.023-.55a2.25 2.25 0 00-2.134 0l-1.022.55m0 0l-4.661 2.51" /></svg>"#.to_string(),
            url: "/console/voicemail".to_string(),
            permission: Some("voicemail.view".to_string()),
        }]
    }
}
