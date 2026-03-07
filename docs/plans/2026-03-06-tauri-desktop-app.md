# 4iiz Tauri Desktop App Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate the 4iiz Leptos/WASM web UI into a Tauri 2 desktop application with native audio capture, real-time AI transcription, and an AI coaching assistant for live calls.

**Architecture:** Tauri 2 wraps the existing Leptos CSR frontend in a native webview. A Rust backend layer manages API connectivity (proxying calls to a remote RustPBX server), native audio capture via `cpal`, real-time transcription via Whisper API streaming, and AI coaching via LLM integration. The frontend communicates with the Tauri backend through Tauri commands (for request/response) and events (for real-time streaming data like transcription and coaching suggestions). All existing Leptos UI code (~8,000 lines, 87 pages, DaisyUI components) is preserved with minimal changes.

**Tech Stack:** Tauri 2.x, Leptos 0.8 (CSR/WASM), DaisyUI 5 + Tailwind v4, cpal (native audio), diesel-async (optional local DB), reqwest (API client), tokio (async runtime), serde (serialization)

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────┐
│                    Tauri Desktop App                      │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │           WebView (Leptos CSR / WASM)              │  │
│  │                                                    │  │
│  │  ┌──────────┐ ┌──────────┐ ┌───────────────────┐  │  │
│  │  │ Activity  │ │ Contacts │ │ AI Coach Panel    │  │  │
│  │  │ Pages     │ │ Numbers  │ │ (new component)   │  │  │
│  │  │ (existing)│ │ Flows    │ │                   │  │  │
│  │  └──────────┘ └──────────┘ └───────────────────┘  │  │
│  │  ┌──────────┐ ┌──────────────────────────────────┐ │  │
│  │  │ Phone    │ │ Live Transcript Panel             │ │  │
│  │  │ Drawer   │ │ (new component)                   │ │  │
│  │  │(upgraded)│ │                                    │ │  │
│  │  └──────────┘ └──────────────────────────────────┘ │  │
│  └──────────┬─────────────────────────┬───────────────┘  │
│             │ Tauri Commands          │ Tauri Events      │
│             │ (request/response)      │ (streaming)       │
│             ▼                         ▼                   │
│  ┌────────────────────────────────────────────────────┐  │
│  │              Tauri Rust Backend                     │  │
│  │                                                    │  │
│  │  ┌──────────────┐  ┌───────────────────────────┐  │  │
│  │  │ API Proxy    │  │ Audio Engine               │  │  │
│  │  │ (reqwest →   │  │ (cpal capture → ring buf   │  │  │
│  │  │  RustPBX     │  │  → chunk → transcription)  │  │  │
│  │  │  server)     │  │                             │  │  │
│  │  └──────────────┘  └───────────────────────────┘  │  │
│  │  ┌──────────────┐  ┌───────────────────────────┐  │  │
│  │  │ Transcription│  │ AI Coach                   │  │  │
│  │  │ Engine       │  │ (transcript → LLM →        │  │  │
│  │  │ (Whisper API │  │  coaching suggestions)     │  │  │
│  │  │  streaming)  │  │                             │  │  │
│  │  └──────────────┘  └───────────────────────────┘  │  │
│  │  ┌──────────────┐  ┌───────────────────────────┐  │  │
│  │  │ Config       │  │ System Integration          │  │  │
│  │  │ (server URL, │  │ (tray, hotkeys,            │  │  │
│  │  │  auth, prefs)│  │  notifications, updater)   │  │  │
│  │  └──────────────┘  └───────────────────────────┘  │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
            │                        │
            ▼                        ▼
   ┌─────────────────┐    ┌──────────────────┐
   │ RustPBX Server   │    │ AI Services       │
   │ (remote)         │    │ - Groq Whisper    │
   │ - REST API       │    │ - Claude API      │
   │ - PostgreSQL     │    │ - Local Whisper   │
   │ - SIP/WebRTC     │    │   (optional)      │
   └─────────────────┘    └──────────────────┘
```

## Key Design Decisions

### 1. Frontend: Keep Leptos, minimal changes
The existing ~8,000 lines of Leptos UI code runs in Tauri's webview identically to a browser. The only change needed is replacing `gloo-net` HTTP calls with Tauri command invocations (via `wasm-bindgen` bridge to `@tauri-apps/api/core`). DaisyUI, Tailwind, leptos-daisyui-rs — all unchanged.

### 2. API Connectivity: Tauri command proxy
Instead of the browser making direct HTTP calls to `/api/v1/*`, the Leptos frontend invokes Tauri commands which proxy to the remote RustPBX server. This keeps auth tokens in Rust (not JS localStorage), supports configurable server URLs, and enables offline caching/retry logic.

### 3. Audio: Dual-stream capture with cpal
`cpal` provides two independent audio streams:
- **Input stream** (microphone) — captures the agent's voice
- **Output loopback** (WASAPI loopback on Windows) — captures the caller's voice from the speaker

These are kept separate for speaker diarization in the transcription pipeline.

### 4. Transcription: Chunked streaming to Whisper API
Audio is buffered in a ring buffer, chunked into ~3-5 second segments, and sent to Groq's Whisper API (or local whisper.cpp for offline). Results stream back as Tauri events.

### 5. AI Coaching: Context-aware LLM suggestions
The transcription stream feeds into a sliding-window context that's sent to Claude API with a coaching system prompt. Suggestions stream back as Tauri events and render in a new coaching panel.

---

## Phase 1: Tauri Scaffolding + Leptos Integration

### Task 1: Create Tauri 2 Project Structure

**Files:**
- Create: `desktop/` (new Tauri project root)
- Create: `desktop/Cargo.toml`
- Create: `desktop/tauri.conf.json`
- Create: `desktop/src/main.rs`
- Create: `desktop/src/lib.rs`
- Create: `desktop/capabilities/default.json`
- Create: `desktop/icons/` (placeholder icons)

**Step 1: Install Tauri CLI**

```bash
cargo install tauri-cli --version "^2"
```

**Step 2: Create the desktop crate**

Create `desktop/Cargo.toml`:

```toml
[package]
name = "iiz-desktop"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
tokio = { version = "1", features = ["full"] }
log = "0.4"
env_logger = "0.11"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

Create `desktop/build.rs`:

```rust
fn main() {
    tauri_build::build()
}
```

Create `desktop/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    iiz_desktop::run()
}
```

Create `desktop/src/lib.rs`:

```rust
mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::api_request,
        ])
        .run(tauri::generate_context!())
        .expect("error running 4iiz desktop");
}
```

Create `desktop/src/commands.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiRequestArgs {
    pub method: String,
    pub path: String,
    pub body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status: u16,
    pub body: String,
}

/// Proxy an API request to the remote RustPBX server.
#[tauri::command]
pub async fn api_request(args: ApiRequestArgs) -> Result<ApiResponse, String> {
    // Placeholder — wired in Task 7
    Ok(ApiResponse {
        status: 200,
        body: r#"{"items":[],"page":1,"per_page":25,"total_items":0,"total_pages":0,"has_prev":false,"has_next":false}"#.to_string(),
    })
}
```

**Step 3: Create Tauri configuration**

Create `desktop/tauri.conf.json`:

```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-config-schema/schema.json",
  "productName": "4iiz",
  "version": "0.1.0",
  "identifier": "com.4iiz.desktop",
  "build": {
    "frontendDist": "../ui/dist",
    "devUrl": "http://localhost:3000",
    "beforeDevCommand": "cd ../ui && trunk serve --port 3000",
    "beforeBuildCommand": "cd ../ui && trunk build --release"
  },
  "app": {
    "title": "4iiz",
    "windows": [
      {
        "title": "4iiz",
        "width": 1440,
        "height": 900,
        "minWidth": 1024,
        "minHeight": 700,
        "center": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

Create `desktop/capabilities/default.json`:

```json
{
  "identifier": "default",
  "description": "Default capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open"
  ]
}
```

**Step 4: Create placeholder icons**

```bash
mkdir -p desktop/icons
# Generate placeholder icons (32x32, 128x128, etc.)
# For now, copy any PNG as placeholder
```

**Step 5: Verify the Tauri project compiles**

```bash
cd desktop && cargo check
```
Expected: Clean compile

**Step 6: Commit**

```bash
git add desktop/
git commit -m "feat: scaffold Tauri 2 desktop project structure"
```

---

### Task 2: Add Tauri IPC Bridge to Leptos Frontend

**Files:**
- Modify: `ui/Cargo.toml` (add optional tauri feature)
- Create: `ui/src/api/tauri_bridge.rs` (wasm-bindgen → Tauri invoke)
- Modify: `ui/src/api/mod.rs` (conditional: use bridge or gloo-net)

**Step 1: Add feature flag and dependencies to ui/Cargo.toml**

Add under `[features]`:

```toml
[features]
default = []
tauri = ["dep:js-sys"]
```

The `js-sys` dep already exists but may need to be optional. The key dependency
is `wasm-bindgen` (already present) for calling JavaScript's Tauri invoke.

**Step 2: Create the Tauri IPC bridge**

Create `ui/src/api/tauri_bridge.rs`:

```rust
//! Bridge from Leptos/WASM to Tauri's invoke() IPC.
//!
//! When compiled with the `tauri` feature, API calls are routed through
//! Tauri commands instead of direct HTTP via gloo-net.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn tauri_invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize)]
struct ApiRequestArgs {
    method: String,
    path: String,
    body: Option<String>,
}

#[derive(Deserialize)]
struct ApiResponse {
    status: u16,
    body: String,
}

/// GET request through Tauri command proxy.
pub async fn api_get<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let args = ApiRequestArgs {
        method: "GET".into(),
        path: format!("/api/v1{}", path),
        body: None,
    };
    let js_args = serde_wasm_bindgen::to_value(&args)
        .map_err(|e| format!("serialize args: {}", e))?;

    let result = tauri_invoke("api_request", js_args).await;

    let resp: ApiResponse = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("deserialize response: {}", e))?;

    if resp.status >= 400 {
        return Err(format!("API error ({}): {}", resp.status, resp.body));
    }

    serde_json::from_str(&resp.body)
        .map_err(|e| format!("parse body: {}", e))
}

/// POST request through Tauri command proxy.
pub async fn api_post<B: Serialize, T: serde::de::DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<T, String> {
    let body_json = serde_json::to_string(body)
        .map_err(|e| format!("serialize body: {}", e))?;
    let args = ApiRequestArgs {
        method: "POST".into(),
        path: format!("/api/v1{}", path),
        body: Some(body_json),
    };
    let js_args = serde_wasm_bindgen::to_value(&args)
        .map_err(|e| format!("serialize args: {}", e))?;

    let result = tauri_invoke("api_request", js_args).await;

    let resp: ApiResponse = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("deserialize response: {}", e))?;

    if resp.status >= 400 {
        return Err(format!("API error ({}): {}", resp.status, resp.body));
    }

    serde_json::from_str(&resp.body)
        .map_err(|e| format!("parse body: {}", e))
}

/// PUT request through Tauri command proxy.
pub async fn api_put<B: Serialize, T: serde::de::DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<T, String> {
    let body_json = serde_json::to_string(body)
        .map_err(|e| format!("serialize body: {}", e))?;
    let args = ApiRequestArgs {
        method: "PUT".into(),
        path: format!("/api/v1{}", path),
        body: Some(body_json),
    };
    let js_args = serde_wasm_bindgen::to_value(&args)
        .map_err(|e| format!("serialize args: {}", e))?;

    let result = tauri_invoke("api_request", js_args).await;

    let resp: ApiResponse = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("deserialize response: {}", e))?;

    if resp.status >= 400 {
        return Err(format!("API error ({}): {}", resp.status, resp.body));
    }

    serde_json::from_str(&resp.body)
        .map_err(|e| format!("parse body: {}", e))
}

/// DELETE request through Tauri command proxy.
pub async fn api_delete<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let args = ApiRequestArgs {
        method: "DELETE".into(),
        path: format!("/api/v1{}", path),
        body: None,
    };
    let js_args = serde_wasm_bindgen::to_value(&args)
        .map_err(|e| format!("serialize args: {}", e))?;

    let result = tauri_invoke("api_request", js_args).await;

    let resp: ApiResponse = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("deserialize response: {}", e))?;

    if resp.status >= 400 {
        return Err(format!("API error ({}): {}", resp.status, resp.body));
    }

    serde_json::from_str(&resp.body)
        .map_err(|e| format!("parse body: {}", e))
}
```

**Step 3: Update api/mod.rs with conditional compilation**

Modify `ui/src/api/mod.rs` to dispatch to either gloo-net or the Tauri bridge:

```rust
pub mod types;

#[cfg(feature = "tauri")]
mod tauri_bridge;

#[cfg(feature = "tauri")]
pub use tauri_bridge::{api_get, api_post, api_put, api_delete};

#[cfg(not(feature = "tauri"))]
mod http;

// Move existing gloo-net functions into http.rs when tauri feature is off
#[cfg(not(feature = "tauri"))]
pub use http::{api_get, api_post, api_put, api_delete};
```

Create `ui/src/api/http.rs` with the existing `api_get`, `api_post`, `api_put`, `api_delete` functions moved from `mod.rs`.

**Step 4: Add serde-wasm-bindgen dependency**

Add to `ui/Cargo.toml` dependencies:

```toml
serde-wasm-bindgen = "0.6"
```

**Step 5: Verify WASM compile with both feature sets**

```bash
cd ui && cargo check --target wasm32-unknown-unknown
cd ui && cargo check --target wasm32-unknown-unknown --features tauri
```

**Step 6: Commit**

```bash
git add ui/
git commit -m "feat(ui): add Tauri IPC bridge with feature-gated API dispatch"
```

---

### Task 3: Wire Tauri Build Pipeline

**Files:**
- Modify: `desktop/tauri.conf.json` (verify dev/build commands)
- Create: `desktop/scripts/dev.sh` (convenience dev script)

**Step 1: Verify Trunk builds for Tauri**

The `desktop/tauri.conf.json` `build.beforeBuildCommand` runs `cd ../ui && trunk build --release`. Trunk needs to build with the `tauri` feature:

Update `desktop/tauri.conf.json` build commands:

```json
{
  "build": {
    "frontendDist": "../ui/dist",
    "devUrl": "http://localhost:3000",
    "beforeDevCommand": "cd ../ui && trunk serve --port 3000 --features tauri",
    "beforeBuildCommand": "cd ../ui && trunk build --release --features tauri"
  }
}
```

Note: Trunk passes `--features` to `cargo build`. Verify by checking Trunk docs.
If Trunk doesn't support `--features`, add to `ui/Trunk.toml`:

```toml
[build]
features = ["tauri"]
```

But this would break the web build. Alternative: use environment variable detection.
Simplest approach: **use a separate `Trunk-desktop.toml`** for the desktop build:

Create `ui/Trunk-desktop.toml`:

```toml
[build]
target = "index.html"
dist = "dist"
filehash = true

[watch]
watch = ["src", "input.css", "index.html"]

[[hooks]]
stage = "pre_build"
command = "npx.cmd"
command_arguments = ["tailwindcss", "-i", "input.css", "-o", "output.css"]

[tools]
wasm_opt = "z"

[build.features]
features = ["tauri"]
```

Update `desktop/tauri.conf.json`:

```json
{
  "build": {
    "frontendDist": "../ui/dist",
    "devUrl": "http://localhost:3000",
    "beforeDevCommand": "cd ../ui && trunk serve --port 3000 --config Trunk-desktop.toml",
    "beforeBuildCommand": "cd ../ui && trunk build --release --config Trunk-desktop.toml"
  }
}
```

**Step 2: Test the full dev cycle**

```bash
cd desktop && cargo tauri dev
```

Expected: Tauri opens a native window, Trunk serves Leptos app inside it.
The app should render but API calls will return placeholder data.

**Step 3: Commit**

```bash
git add desktop/ ui/Trunk-desktop.toml
git commit -m "feat: wire Tauri build pipeline with Trunk desktop config"
```

---

## Phase 2: API Connectivity Layer

### Task 4: Implement Server Connection Configuration

**Files:**
- Create: `desktop/src/config.rs`
- Modify: `desktop/src/lib.rs` (add config state)

**Step 1: Create config module**

Create `desktop/src/config.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Base URL of the RustPBX server (e.g. "https://74.207.251.126:8443")
    pub server_url: String,
    /// JWT auth token (persisted between sessions)
    pub auth_token: Option<String>,
    /// Groq API key for Whisper transcription
    pub groq_api_key: Option<String>,
    /// Anthropic API key for AI coaching
    pub anthropic_api_key: Option<String>,
    /// Audio input device name (None = system default)
    pub audio_input_device: Option<String>,
    /// Audio output device name (None = system default)
    pub audio_output_device: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_url: "https://localhost:8443".into(),
            auth_token: None,
            groq_api_key: None,
            anthropic_api_key: None,
            audio_input_device: None,
            audio_output_device: None,
        }
    }
}

/// Thread-safe config state managed by Tauri.
pub struct ConfigState(pub RwLock<AppConfig>);

impl ConfigState {
    pub fn load() -> Self {
        let path = config_path();
        let config = if path.exists() {
            let data = std::fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            AppConfig::default()
        };
        Self(RwLock::new(config))
    }

    pub fn save(&self) -> Result<(), String> {
        let config = self.0.read().map_err(|e| e.to_string())?;
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let data = serde_json::to_string_pretty(&*config).map_err(|e| e.to_string())?;
        std::fs::write(&path, data).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("4iiz")
        .join("config.json")
}
```

**Step 2: Add config commands**

Add to `desktop/src/commands.rs`:

```rust
use crate::config::ConfigState;

#[tauri::command]
pub async fn get_config(
    state: tauri::State<'_, ConfigState>,
) -> Result<crate::config::AppConfig, String> {
    let config = state.0.read().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub async fn set_server_url(
    url: String,
    state: tauri::State<'_, ConfigState>,
) -> Result<(), String> {
    {
        let mut config = state.0.write().map_err(|e| e.to_string())?;
        config.server_url = url;
    }
    state.save()
}

#[tauri::command]
pub async fn set_auth_token(
    token: String,
    state: tauri::State<'_, ConfigState>,
) -> Result<(), String> {
    {
        let mut config = state.0.write().map_err(|e| e.to_string())?;
        config.auth_token = Some(token);
    }
    state.save()
}
```

**Step 3: Register config state in lib.rs**

Update `desktop/src/lib.rs`:

```rust
mod commands;
mod config;

use config::ConfigState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(ConfigState::load())
        .invoke_handler(tauri::generate_handler![
            commands::api_request,
            commands::get_config,
            commands::set_server_url,
            commands::set_auth_token,
        ])
        .run(tauri::generate_context!())
        .expect("error running 4iiz desktop");
}
```

**Step 4: Add dirs dependency**

Add to `desktop/Cargo.toml`:

```toml
dirs = "6"
```

**Step 5: Verify compile + commit**

```bash
cd desktop && cargo check
git add desktop/
git commit -m "feat(desktop): add persistent server connection configuration"
```

---

### Task 5: Implement API Proxy Command with reqwest

**Files:**
- Modify: `desktop/src/commands.rs` (real HTTP proxy implementation)

**Step 1: Replace placeholder api_request with real reqwest proxy**

Replace the `api_request` function in `desktop/src/commands.rs`:

```rust
use crate::config::ConfigState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiRequestArgs {
    pub method: String,
    pub path: String,
    pub body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status: u16,
    pub body: String,
}

/// Proxy an API request to the remote RustPBX server.
/// All API calls from the Leptos frontend route through here.
#[tauri::command]
pub async fn api_request(
    args: ApiRequestArgs,
    state: tauri::State<'_, ConfigState>,
) -> Result<ApiResponse, String> {
    let (server_url, auth_token) = {
        let config = state.0.read().map_err(|e| e.to_string())?;
        (config.server_url.clone(), config.auth_token.clone())
    };

    let url = format!("{}{}", server_url, args.path);

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true) // Self-signed certs in dev
        .build()
        .map_err(|e| e.to_string())?;

    let mut req = match args.method.as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        _ => return Err(format!("Unsupported method: {}", args.method)),
    };

    // Add auth header
    if let Some(token) = &auth_token {
        req = req.header("Authorization", format!("Bearer {}", token));
    }

    // Add body for POST/PUT
    if let Some(body) = &args.body {
        req = req.header("Content-Type", "application/json").body(body.clone());
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    let body = resp.text().await.map_err(|e| e.to_string())?;

    Ok(ApiResponse { status, body })
}
```

**Step 2: Verify compile + commit**

```bash
cd desktop && cargo check
git add desktop/src/commands.rs
git commit -m "feat(desktop): implement real API proxy via reqwest"
```

---

### Task 6: Add Connection Health Check Command

**Files:**
- Modify: `desktop/src/commands.rs` (add health check)

**Step 1: Add health check command**

```rust
#[tauri::command]
pub async fn check_server_health(
    state: tauri::State<'_, ConfigState>,
) -> Result<bool, String> {
    let server_url = {
        let config = state.0.read().map_err(|e| e.to_string())?;
        config.server_url.clone()
    };

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    match client.get(format!("{}/api/v1/health", server_url)).send().await {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(_) => Ok(false),
    }
}
```

**Step 2: Register in lib.rs invoke_handler + commit**

```bash
git add desktop/
git commit -m "feat(desktop): add server health check command"
```

---

## Phase 3: Native Audio Pipeline

### Task 7: Add cpal Audio Device Enumeration

**Files:**
- Create: `desktop/src/audio.rs`
- Modify: `desktop/Cargo.toml` (add cpal)
- Modify: `desktop/src/lib.rs` (register audio module)

**Step 1: Add cpal dependency**

Add to `desktop/Cargo.toml`:

```toml
cpal = "0.15"
```

**Step 2: Create audio module with device enumeration**

Create `desktop/src/audio.rs`:

```rust
use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AudioDevice {
    pub name: String,
    pub is_input: bool,
    pub is_default: bool,
}

#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();
    let mut devices = Vec::new();

    let default_input = host.default_input_device()
        .and_then(|d| d.name().ok());
    let default_output = host.default_output_device()
        .and_then(|d| d.name().ok());

    // Input devices
    if let Ok(inputs) = host.input_devices() {
        for device in inputs {
            if let Ok(name) = device.name() {
                devices.push(AudioDevice {
                    is_default: default_input.as_deref() == Some(&name),
                    name,
                    is_input: true,
                });
            }
        }
    }

    // Output devices
    if let Ok(outputs) = host.output_devices() {
        for device in outputs {
            if let Ok(name) = device.name() {
                devices.push(AudioDevice {
                    is_default: default_output.as_deref() == Some(&name),
                    name,
                    is_input: false,
                });
            }
        }
    }

    Ok(devices)
}
```

**Step 3: Register commands + verify + commit**

```bash
cd desktop && cargo check
git add desktop/
git commit -m "feat(desktop): add cpal audio device enumeration"
```

---

### Task 8: Implement Microphone Capture Stream

**Files:**
- Modify: `desktop/src/audio.rs` (add capture engine)
- Create: `desktop/src/audio/capture.rs`

**Step 1: Create audio capture engine**

Create `desktop/src/audio/mod.rs` (refactor from `audio.rs`):

```rust
pub mod capture;
pub mod devices;
```

Create `desktop/src/audio/devices.rs` (move device enumeration here).

Create `desktop/src/audio/capture.rs`:

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// Audio sample rate for capture (16kHz is optimal for Whisper)
pub const SAMPLE_RATE: u32 = 16000;
/// Channels: mono for speech
pub const CHANNELS: u16 = 1;

/// Ring buffer of f32 audio samples with broadcast notification.
pub struct AudioCapture {
    /// Sender for audio chunk events (16-bit PCM chunks)
    sender: broadcast::Sender<Vec<i16>>,
    /// Active cpal stream (kept alive)
    _stream: Option<cpal::Stream>,
    /// Accumulator buffer
    buffer: Arc<Mutex<Vec<f32>>>,
    /// Chunk size in samples (~3 seconds at 16kHz)
    chunk_size: usize,
}

impl AudioCapture {
    pub fn new(chunk_duration_secs: f32) -> Self {
        let (sender, _) = broadcast::channel(32);
        let chunk_size = (SAMPLE_RATE as f32 * chunk_duration_secs) as usize;
        Self {
            sender,
            _stream: None,
            buffer: Arc::new(Mutex::new(Vec::with_capacity(chunk_size * 2))),
            chunk_size,
        }
    }

    /// Subscribe to audio chunks (each chunk is ~3s of 16-bit PCM @ 16kHz mono)
    pub fn subscribe(&self) -> broadcast::Receiver<Vec<i16>> {
        self.sender.subscribe()
    }

    /// Start capturing from the default input device.
    pub fn start_mic_capture(&mut self) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or("No input device found")?;

        let config = cpal::StreamConfig {
            channels: CHANNELS,
            sample_rate: cpal::SampleRate(SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Default,
        };

        let buffer = self.buffer.clone();
        let sender = self.sender.clone();
        let chunk_size = self.chunk_size;

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buf = buffer.lock().unwrap();
                buf.extend_from_slice(data);

                // When we have enough samples, emit a chunk
                while buf.len() >= chunk_size {
                    let chunk_f32: Vec<f32> = buf.drain(..chunk_size).collect();
                    // Convert f32 → i16 PCM
                    let chunk_i16: Vec<i16> = chunk_f32.iter()
                        .map(|&s| (s * i16::MAX as f32) as i16)
                        .collect();
                    let _ = sender.send(chunk_i16);
                }
            },
            |err| {
                log::error!("Audio capture error: {}", err);
            },
            None,
        ).map_err(|e| format!("Failed to build input stream: {}", e))?;

        stream.play().map_err(|e| format!("Failed to start stream: {}", e))?;
        self._stream = Some(stream);

        Ok(())
    }

    /// Stop capturing.
    pub fn stop(&mut self) {
        self._stream = None;
        if let Ok(mut buf) = self.buffer.lock() {
            buf.clear();
        }
    }
}
```

**Step 2: Create Tauri commands for audio control**

Add to commands:

```rust
use crate::audio::capture::AudioCapture;
use std::sync::Mutex;

pub struct AudioState(pub Mutex<AudioCapture>);

#[tauri::command]
pub fn start_mic_capture(state: tauri::State<'_, AudioState>) -> Result<(), String> {
    let mut capture = state.0.lock().map_err(|e| e.to_string())?;
    capture.start_mic_capture()
}

#[tauri::command]
pub fn stop_mic_capture(state: tauri::State<'_, AudioState>) -> Result<(), String> {
    let mut capture = state.0.lock().map_err(|e| e.to_string())?;
    capture.stop();
    Ok(())
}
```

**Step 3: Verify compile + commit**

```bash
cd desktop && cargo check
git add desktop/
git commit -m "feat(desktop): implement microphone capture with cpal"
```

---

### Task 9: Add Audio Event Streaming to Frontend

**Files:**
- Modify: `desktop/src/lib.rs` (spawn audio→event bridge)
- Modify: `desktop/src/audio/capture.rs` (emit Tauri events)

**Step 1: Bridge audio chunks to Tauri events**

In `desktop/src/lib.rs`, after starting audio capture, spawn a tokio task that
reads from the broadcast receiver and emits Tauri events:

```rust
use tauri::Emitter;

// In the setup closure:
.setup(|app| {
    let handle = app.handle().clone();

    // Spawn audio→frontend event bridge
    let audio_state = app.state::<AudioState>();
    let mut rx = audio_state.0.lock().unwrap().subscribe();

    tauri::async_runtime::spawn(async move {
        while let Ok(chunk) = rx.recv().await {
            // Emit audio level (RMS) for VU meter, not raw PCM
            let rms = (chunk.iter()
                .map(|&s| (s as f64).powi(2))
                .sum::<f64>() / chunk.len() as f64)
                .sqrt();
            let _ = handle.emit("audio-level", rms);
        }
    });

    Ok(())
})
```

The raw PCM chunks are NOT sent to the frontend — they stay in Rust for
transcription. Only derived data (levels, transcription text) goes to the UI.

**Step 2: Commit**

```bash
git add desktop/
git commit -m "feat(desktop): stream audio levels to frontend via Tauri events"
```

---

## Phase 4: Real-Time Transcription Engine

### Task 10: Implement Whisper API Streaming Client

**Files:**
- Create: `desktop/src/transcription.rs`
- Modify: `desktop/Cargo.toml` (add multipart form deps)

**Step 1: Create transcription module**

Create `desktop/src/transcription.rs`:

```rust
use reqwest::multipart;
use serde::Deserialize;
use tokio::sync::broadcast;

#[derive(Debug, Clone, serde::Serialize)]
pub struct TranscriptSegment {
    pub speaker: String, // "agent" or "caller"
    pub text: String,
    pub timestamp_secs: f64,
}

pub struct TranscriptionEngine {
    groq_api_key: String,
    sender: broadcast::Sender<TranscriptSegment>,
}

#[derive(Deserialize)]
struct WhisperResponse {
    text: String,
}

impl TranscriptionEngine {
    pub fn new(groq_api_key: String) -> Self {
        let (sender, _) = broadcast::channel(64);
        Self { groq_api_key, sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<TranscriptSegment> {
        self.sender.subscribe()
    }

    /// Transcribe a chunk of 16-bit PCM audio (16kHz mono).
    /// Sends the chunk to Groq Whisper API and emits the result.
    pub async fn transcribe_chunk(
        &self,
        pcm_i16: &[i16],
        speaker: &str,
        timestamp_secs: f64,
    ) -> Result<(), String> {
        // Convert i16 PCM to WAV in memory
        let wav_data = pcm_to_wav(pcm_i16, 16000, 1);

        let client = reqwest::Client::new();
        let part = multipart::Part::bytes(wav_data)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| e.to_string())?;

        let form = multipart::Form::new()
            .text("model", "whisper-large-v3")
            .text("language", "en")
            .text("response_format", "json")
            .part("file", part);

        let resp = client
            .post("https://api.groq.com/openai/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", self.groq_api_key))
            .multipart(form)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let whisper: WhisperResponse = resp.json().await.map_err(|e| e.to_string())?;

        if !whisper.text.trim().is_empty() {
            let segment = TranscriptSegment {
                speaker: speaker.to_string(),
                text: whisper.text.trim().to_string(),
                timestamp_secs,
            };
            let _ = self.sender.send(segment);
        }

        Ok(())
    }
}

/// Encode i16 PCM samples as a WAV file in memory.
fn pcm_to_wav(samples: &[i16], sample_rate: u32, channels: u16) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let file_len = 36 + data_len;
    let byte_rate = sample_rate * channels as u32 * 2;
    let block_align = channels * 2;

    let mut buf = Vec::with_capacity(44 + samples.len() * 2);
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_len.to_le_bytes());
    buf.extend_from_slice(b"WAVE");
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    buf.extend_from_slice(&1u16.to_le_bytes());  // PCM format
    buf.extend_from_slice(&channels.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&block_align.to_le_bytes());
    buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_len.to_le_bytes());
    for &s in samples {
        buf.extend_from_slice(&s.to_le_bytes());
    }
    buf
}
```

**Step 2: Wire transcription to audio capture pipeline**

In the Tauri setup, spawn a task that reads audio chunks and feeds them to the
transcription engine, then emits transcript segments as Tauri events:

```rust
// In setup closure:
let mut audio_rx = audio_state.subscribe();
let transcription = TranscriptionEngine::new(groq_key);
let mut transcript_rx = transcription.subscribe();
let handle2 = app.handle().clone();

// Audio → Transcription
tauri::async_runtime::spawn(async move {
    let mut elapsed = 0.0f64;
    while let Ok(chunk) = audio_rx.recv().await {
        let duration = chunk.len() as f64 / 16000.0;
        let _ = transcription.transcribe_chunk(&chunk, "agent", elapsed).await;
        elapsed += duration;
    }
});

// Transcription → Frontend
tauri::async_runtime::spawn(async move {
    while let Ok(segment) = transcript_rx.recv().await {
        let _ = handle2.emit("transcript-segment", &segment);
    }
});
```

**Step 3: Verify compile + commit**

```bash
cd desktop && cargo check
git add desktop/
git commit -m "feat(desktop): implement Whisper API transcription engine"
```

---

### Task 11: Create Live Transcript UI Component

**Files:**
- Create: `ui/src/components/live_transcript.rs`
- Modify: `ui/src/components/mod.rs` (register)
- Modify: `ui/src/sections/activities.rs` (integrate into CallsPage)

**Step 1: Create the transcript panel component**

This component listens for `transcript-segment` Tauri events and renders
a scrolling transcript with speaker labels.

Create `ui/src/components/live_transcript.rs`:

```rust
use leptos::prelude::*;
use leptos_icons::Icon;

#[derive(Clone, Debug)]
struct TranscriptLine {
    speaker: String,
    text: String,
}

#[component]
pub fn LiveTranscriptPanel(
    #[prop(into)] visible: Signal<bool>,
) -> impl IntoView {
    let lines = RwSignal::new(Vec::<TranscriptLine>::new());

    // Listen for Tauri events (only in tauri mode)
    #[cfg(feature = "tauri")]
    {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        // Set up Tauri event listener on mount
        Effect::new(move || {
            // JS: window.__TAURI__.event.listen('transcript-segment', callback)
            // This will be wired via a small JS shim
        });
    }

    view! {
        <Show when=move || visible.get()>
            <div class="w-80 border-l border-gray-200 bg-white flex flex-col h-full">
                <div class="h-10 border-b border-gray-200 flex items-center px-3 gap-2 flex-shrink-0">
                    <span class="w-4 h-4 inline-flex text-iiz-cyan"><Icon icon=icondata::BsMic /></span>
                    <span class="text-sm font-semibold text-gray-700">"Live Transcript"</span>
                    <div class="flex-1"></div>
                    <span class="w-2 h-2 rounded-full bg-green-400 animate-pulse"></span>
                </div>
                <div class="flex-1 overflow-y-auto p-3 space-y-2 text-sm">
                    {move || lines.get().iter().map(|line| {
                        let is_agent = line.speaker == "agent";
                        let label_class = if is_agent { "text-iiz-cyan" } else { "text-iiz-orange" };
                        let speaker = line.speaker.clone();
                        let text = line.text.clone();
                        view! {
                            <div>
                                <span class={format!("font-semibold text-xs {}", label_class)}>
                                    {speaker}": "
                                </span>
                                <span class="text-gray-700">{text}</span>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </Show>
    }
}
```

**Step 2: Register + commit**

```bash
git add ui/src/components/
git commit -m "feat(ui): add LiveTranscriptPanel component"
```

---

## Phase 5: AI Coaching Assistant

### Task 12: Implement Coaching Engine

**Files:**
- Create: `desktop/src/coaching.rs`

**Step 1: Create coaching engine**

The coaching engine maintains a sliding window of transcript context and
periodically queries Claude API for response suggestions.

Create `desktop/src/coaching.rs`:

```rust
use serde::Serialize;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize)]
pub struct CoachingSuggestion {
    pub suggestion_type: String, // "response", "question", "objection_handling", "closing"
    pub text: String,
    pub confidence: f32,
}

pub struct CoachingEngine {
    anthropic_api_key: String,
    sender: broadcast::Sender<CoachingSuggestion>,
    transcript_context: Vec<String>,
    max_context_lines: usize,
}

impl CoachingEngine {
    pub fn new(anthropic_api_key: String) -> Self {
        let (sender, _) = broadcast::channel(32);
        Self {
            anthropic_api_key,
            sender,
            transcript_context: Vec::new(),
            max_context_lines: 50,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<CoachingSuggestion> {
        self.sender.subscribe()
    }

    /// Add a transcript line and potentially trigger coaching.
    /// Only triggers after caller speech (agent needs help responding).
    pub async fn on_transcript(&mut self, speaker: &str, text: &str) {
        self.transcript_context.push(format!("{}: {}", speaker, text));

        // Keep sliding window
        if self.transcript_context.len() > self.max_context_lines {
            self.transcript_context.remove(0);
        }

        // Only generate coaching after caller speaks
        if speaker == "caller" {
            if let Err(e) = self.generate_suggestion().await {
                log::warn!("Coaching error: {}", e);
            }
        }
    }

    async fn generate_suggestion(&self) -> Result<(), String> {
        let transcript = self.transcript_context.join("\n");

        let body = serde_json::json!({
            "model": "claude-sonnet-4-5-20250514",
            "max_tokens": 200,
            "system": "You are a real-time call coaching assistant for a law firm intake specialist. Based on the live call transcript, provide a brief, actionable suggestion for what the agent should say or ask next. Focus on: qualifying the lead, gathering case details, showing empathy, and scheduling a consultation. Keep suggestions under 2 sentences. Format: just the suggested response text, nothing else.",
            "messages": [
                {
                    "role": "user",
                    "content": format!("Live call transcript so far:\n\n{}\n\nWhat should the agent say next?", transcript)
                }
            ]
        });

        let client = reqwest::Client::new();
        let resp = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.anthropic_api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

        if let Some(text) = json["content"][0]["text"].as_str() {
            let suggestion = CoachingSuggestion {
                suggestion_type: "response".into(),
                text: text.trim().to_string(),
                confidence: 0.8,
            };
            let _ = self.sender.send(suggestion);
        }

        Ok(())
    }
}
```

**Step 2: Wire coaching events to frontend**

In the Tauri setup, subscribe to transcript segments, feed them to the coaching
engine, and emit coaching suggestions as Tauri events:

```rust
// Transcript → Coaching → Frontend
let mut transcript_rx2 = transcription.subscribe();
let mut coaching = CoachingEngine::new(anthropic_key);
let mut coaching_rx = coaching.subscribe();
let handle3 = app.handle().clone();

tauri::async_runtime::spawn(async move {
    while let Ok(segment) = transcript_rx2.recv().await {
        coaching.on_transcript(&segment.speaker, &segment.text).await;
    }
});

tauri::async_runtime::spawn(async move {
    while let Ok(suggestion) = coaching_rx.recv().await {
        let _ = handle3.emit("coaching-suggestion", &suggestion);
    }
});
```

**Step 3: Commit**

```bash
git add desktop/
git commit -m "feat(desktop): implement AI coaching engine with Claude API"
```

---

### Task 13: Create AI Coach UI Panel

**Files:**
- Create: `ui/src/components/coaching_panel.rs`
- Modify: `ui/src/components/mod.rs`

**Step 1: Create coaching panel component**

Create `ui/src/components/coaching_panel.rs`:

```rust
use leptos::prelude::*;
use leptos_icons::Icon;

#[derive(Clone, Debug)]
struct Suggestion {
    text: String,
    suggestion_type: String,
}

#[component]
pub fn CoachingPanel(
    #[prop(into)] visible: Signal<bool>,
) -> impl IntoView {
    let suggestions = RwSignal::new(Vec::<Suggestion>::new());

    view! {
        <Show when=move || visible.get()>
            <div class="w-80 border-l border-gray-200 bg-gradient-to-b from-blue-50 to-white flex flex-col h-full">
                <div class="h-10 border-b border-gray-200 flex items-center px-3 gap-2 flex-shrink-0 bg-white">
                    <span class="w-4 h-4 inline-flex text-purple-600"><Icon icon=icondata::BsLightbulb /></span>
                    <span class="text-sm font-semibold text-gray-700">"AI Coach"</span>
                    <div class="flex-1"></div>
                    <span class="badge badge-xs badge-primary">"Live"</span>
                </div>
                <div class="flex-1 overflow-y-auto p-3 space-y-3">
                    {move || {
                        let items = suggestions.get();
                        if items.is_empty() {
                            view! {
                                <div class="text-center text-gray-400 text-sm mt-8">
                                    <span class="w-8 h-8 inline-flex text-gray-300 mb-2"><Icon icon=icondata::BsChatDots /></span>
                                    <div>"Suggestions will appear here"</div>
                                    <div class="text-xs mt-1">"during an active call"</div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div>
                                    {items.iter().rev().take(5).map(|s| {
                                        let text = s.text.clone();
                                        view! {
                                            <div class="bg-white rounded-lg shadow-sm border border-purple-100 p-3">
                                                <div class="text-xs text-purple-500 font-medium mb-1">"Suggested Response"</div>
                                                <div class="text-sm text-gray-800 leading-relaxed">{text}</div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </Show>
    }
}
```

**Step 2: Register + commit**

```bash
git add ui/src/components/
git commit -m "feat(ui): add AI CoachingPanel component"
```

---

## Phase 6: Desktop Integration

### Task 14: Add System Tray with Call Status

**Files:**
- Modify: `desktop/src/lib.rs` (add tray setup)

**Step 1: Configure system tray**

```rust
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState};
use tauri::menu::{Menu, MenuItem};

// In setup:
let quit = MenuItem::with_id(app, "quit", "Quit 4iiz", true, None::<&str>)?;
let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
let menu = Menu::with_items(app, &[&show, &quit])?;

let _tray = TrayIconBuilder::new()
    .menu(&menu)
    .tooltip("4iiz - Not in call")
    .on_menu_event(|app, event| {
        match event.id.as_ref() {
            "quit" => app.exit(0),
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        }
    })
    .build(app)?;
```

**Step 2: Commit**

```bash
git add desktop/
git commit -m "feat(desktop): add system tray with call status"
```

---

### Task 15: Add Global Hotkeys

**Files:**
- Modify: `desktop/Cargo.toml` (add global-shortcut plugin)
- Modify: `desktop/src/lib.rs`

**Step 1: Add global shortcut plugin**

```toml
# desktop/Cargo.toml
tauri-plugin-global-shortcut = "2"
```

```rust
// In lib.rs builder:
.plugin(tauri_plugin_global_shortcut::Builder::new().build())
```

Register hotkeys in setup:

```rust
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

app.global_shortcut().on_shortcut(
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyM),
    |app, shortcut, event| {
        if event.state == ShortcutState::Pressed {
            let _ = app.emit("hotkey-mute-toggle", ());
        }
    },
)?;
```

**Step 2: Commit**

```bash
git add desktop/
git commit -m "feat(desktop): add global hotkeys for call control"
```

---

### Task 16: Add Desktop Notifications

**Files:**
- Modify: `desktop/Cargo.toml` (add notification plugin)
- Add notification commands

**Step 1: Add notification plugin**

```toml
tauri-plugin-notification = "2"
```

```rust
.plugin(tauri_plugin_notification::init())
```

**Step 2: Commit**

```bash
git add desktop/
git commit -m "feat(desktop): add desktop notification support"
```

---

### Task 17: Configure Auto-Updater

**Files:**
- Modify: `desktop/Cargo.toml`
- Modify: `desktop/tauri.conf.json`

**Step 1: Add updater plugin**

```toml
tauri-plugin-updater = "2"
```

Configure in `tauri.conf.json`:

```json
{
  "plugins": {
    "updater": {
      "endpoints": ["https://releases.4iiz.com/{{target}}/{{arch}}/{{current_version}}"],
      "pubkey": "<your-public-key>"
    }
  }
}
```

**Step 2: Commit**

```bash
git add desktop/
git commit -m "feat(desktop): configure auto-updater"
```

---

### Task 18: Build Installer Packages

**Files:**
- Modify: `desktop/tauri.conf.json` (bundle config)

**Step 1: Configure Windows MSI + NSIS installers**

```json
{
  "bundle": {
    "active": true,
    "targets": ["msi", "nsis"],
    "icon": ["icons/icon.ico"],
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": ""
    }
  }
}
```

**Step 2: Build release**

```bash
cd desktop && cargo tauri build
```

Output: `desktop/target/release/bundle/msi/4iiz_0.1.0_x64_en-US.msi`

**Step 3: Commit**

```bash
git add desktop/
git commit -m "feat(desktop): configure installer packaging (MSI + NSIS)"
```

---

## Execution Order & Dependencies

```
Phase 1 (scaffolding):  Task 1 → Task 2 → Task 3
Phase 2 (API):          Task 4 → Task 5 → Task 6
Phase 3 (audio):        Task 7 → Task 8 → Task 9
Phase 4 (transcription):Task 10 → Task 11
Phase 5 (coaching):     Task 12 → Task 13
Phase 6 (desktop):      Task 14, 15, 16, 17, 18 (parallel)
```

Phases 1-2 can run independently from Phases 3-5.
Phase 6 tasks are all independent of each other.

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| WASAPI loopback capture (caller audio) | High | Windows only; may need virtual audio cable on macOS. Defer macOS loopback to future phase. |
| Whisper API latency | Medium | Use Groq (fastest Whisper endpoint ~0.5s). Fallback: local whisper.cpp for offline. |
| Coaching LLM latency | Medium | Use Claude Haiku for <1s responses. Show "thinking..." indicator. |
| Tauri + Trunk build pipeline | Low | Well-documented path; Leptos CSR in webview is proven. |
| cpal device selection on diverse hardware | Medium | Default device first; custom selection as settings feature. |
| WASM↔Tauri IPC overhead | Low | Only metadata crosses the bridge; audio stays in Rust. |

## What Stays Unchanged

- All 87 UI pages (Leptos components)
- DaisyUI 5 + Tailwind v4 styling
- leptos-daisyui-rs component library
- API response types (`ui/src/api/types.rs`)
- CallDetailPanel, FilterBar layout
- input.css custom styles and animations
- All backend RustPBX server code
- PostgreSQL schema and Diesel models
- JWT authentication flow (token stored in config instead of localStorage)
