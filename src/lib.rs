pub mod addons;
pub mod app;
pub mod backup;
pub mod call;
pub mod callrecord;
pub mod config;
#[cfg(feature = "console")]
pub mod console;
pub mod fixtures;
pub mod handler;
pub mod license;
pub mod media;
pub mod models;
pub mod preflight;
pub mod proxy;
pub mod services;
pub mod sipflow;
pub mod storage;
pub mod utils;
pub mod version; // Admin console
pub mod voicemail;

// AI Voice Agent modules (from active-call)
pub mod event;
pub mod net_tool;
pub mod synthesis;
pub mod transcription;
pub mod playbook;
#[cfg(feature = "offline")]
pub mod offline;
pub mod useragent;
