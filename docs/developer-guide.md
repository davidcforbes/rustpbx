# RustPBX Developer Guide

This guide covers building, running, testing, and extending RustPBX. For
architecture details see [architecture.md](architecture.md); for config options see
[configuration-reference.md](configuration-reference.md); for production deployment
see [deployment-guide.md](deployment-guide.md).

---

## 1. Repository Structure Walkthrough

### `src/proxy/` -- SIP Proxy and Call Handling

The SIP processing core. Every SIP request flows through a configurable pipeline of
`ProxyModule` implementations defined in `mod.rs`. Key files:

- **`server.rs`** -- `SipServer` / `SipServerBuilder` that binds UDP/TCP/TLS/WS transports and runs the module pipeline
- **`acl.rs`** -- IP allow/deny rules with CIDR matching
- **`auth.rs`** -- SIP Digest MD5 authentication with trunk credential bypass
- **`registrar.rs`** -- REGISTER handling (Contact binding, GRUU, WebRTC detection)
- **`presence.rs`** -- Online/offline status tracking
- **`call.rs`** -- INVITE handling, route matching, trunk selection, B2BUA session creation
- **`nat.rs`** -- `NatInspector` that rewrites Contact headers for NAT traversal
- **`trunk_register.rs`** -- Upstream trunk registration for inbound PSTN delivery
- **`ws.rs`** -- WebSocket SIP transport handler
- **`data.rs`** -- `ProxyDataContext` (shared state: locator, user backends, call registry)
- **`proxy_call/`** -- B2BUA session management: `session.rs` (two dialogs + media bridge), `state.rs` (`SessionAction`, `ProxyCallPhase`), `media_bridge.rs`, `media_peer.rs`
- **`routing/`** -- `TrunkConfig`, `RouteRule`, `MatchConditions`, route matching logic
- **`locator*.rs`** -- Registration binding lookup (memory, database, webhook)
- **`user*.rs`** -- SIP authentication backends (memory, database, extension, HTTP, plain file)
- **`tests/`** -- Proxy module unit tests (ACL, auth, registrar, B2BUA flow, CDR, etc.)

### `src/media/` -- RTP/WebRTC Media

- **`mod.rs`** -- `Track` trait and implementations: `RtpTrack` (plain RTP), `RtcTrack` (WebRTC/DTLS-SRTP), `FileTrack` (playback), `MediaStream`
- **`recorder.rs`** -- Stereo WAV recorder with per-leg decoding and gain control
- **`transcoder.rs`** -- Audio codec transcoding between legs
- **`call_quality.rs`** -- Per-leg RTP quality metrics (packet loss, jitter, MOS estimation)
- **`negotiate.rs`** -- SDP codec negotiation helpers
- **`audio_source.rs`** -- Dynamic audio source switching
- **`wav_writer.rs`** -- Low-level WAV file writer

### `src/console/` -- Admin Web UI

Feature-gated behind `"console"`. Uses axum + minijinja (Jinja2) + Alpine.js + Tailwind CSS.

- **`mod.rs`** -- `ConsoleState`, template rendering (`state.render()`), session management
- **`auth.rs`** -- Session-based authentication with HMAC-SHA256 cookies
- **`middleware.rs`** -- `AuthRequired` extractor, `RenderTemplate`
- **`handlers/`** -- One file per section: `dashboard.rs`, `extension.rs`, `sip_trunk.rs`, `routing.rs`, `call_record.rs`, `call_control.rs`, `diagnostics.rs`, `sipflow.rs`, `presence.rs`, `voicemail.rs`, `setting.rs`, `user.rs`, `addons.rs`

### `src/models/` -- SeaORM Database Models

Each file defines an entity struct (with `DeriveEntityModel`) and a `Migration` impl:

- `user.rs`, `extension.rs`, `sip_trunk.rs`, `routing.rs`, `call_record.rs`, `presence.rs`, `department.rs`, `voicemail.rs`
- `migration.rs` -- `Migrator` struct with ordered migration list (migrations run at startup)
- Additional migration-only files for schema evolution (e.g., `add_quality_columns.rs`)

### `src/addons/` -- Optional Features

Trait-based plugin system. Each addon implements the `Addon` trait from `mod.rs`:

- **`registry.rs`** -- `AddonRegistry` manages addon lifecycle, hooks, and router merging
- **`acme/`** -- Let's Encrypt ACME certificate automation
- **`archive/`** -- Call record archival and retention
- **`transcript/`** -- Call transcription (external ASR commands)
- **`queue/`** -- Call queue with hold music and dial strategies
- **`wholesale/`**, **`endpoint_manager/`**, **`enterprise_auth/`** -- Commercial addons

### `src/handler/` -- AMI API Handlers

- **`ami.rs`** -- REST management API at `/ami/v1/*` (call control, extension management, system status)
- **`middleware/`** -- Request logging, client IP extraction, AMI token authentication

### `templates/console/` -- HTML Templates

minijinja templates extending `layout.html`. Each page uses Alpine.js `x-data` components
that fetch JSON from companion `/data` endpoints. Files: `dashboard.html`, `extensions.html`,
`extension_detail.html`, `sip_trunk.html`, `routing.html`, `call_records.html`, etc.

### Other Directories

- **`src/callrecord/`** -- CDR processing pipeline, `CallRecordHook` trait, SIP flow capture
- **`src/call/`** -- Call planning data structures (`Dialplan`, `Location`, `TransactionCookie`)
- **`src/sipflow/`** -- SIP message tracing items and local/remote backends
- **`src/storage/`** -- Unified object storage (local, S3, GCP, Azure) via `object_store`
- **`static/`** -- JS files, browser softphones (`phone_jssip.html`, `phone_sipjs.html`)
- **`config/`** -- Runtime dirs: `certs/`, `recorders/`, `sounds/`, `trunks/`, `routes/`, `queue/`
- **`tests/`** -- Python integration tests (tiered L0-L9), SIP test utilities, Docker compose

---

## 2. Build Prerequisites

**Rust toolchain:** Edition 2024 (Rust 1.85+). Install via [rustup](https://rustup.rs/).

**System dependencies:**

| Dependency | Platform | Purpose | Install |
|-----------|----------|---------|---------|
| build-essential / C++ toolchain | Linux / Windows | Native dependency compilation | `apt install build-essential` / VS Build Tools |
| pkg-config | Linux | Library discovery | `apt install pkg-config` |
| cmake | Linux / macOS | Some native deps | `apt install cmake` / `brew install cmake` |
| Python 3.10+ | Optional | Integration tests | `apt install python3` |

TLS uses rustls (bundled) -- no system OpenSSL required.

**Local path dependencies:** `Cargo.toml` may reference `rsipstack` and `rustrtc` as
local paths for development. For a clean build against published crates, switch to
version dependencies (`rsipstack = "0.4.7"`, `rustrtc = "0.3.21"`).

---

## 3. Building and Running

### Building

```bash
cargo build                    # Debug build (fast compile)
cargo build --release          # Release build (optimized)
cargo build --release --features "opus,console,addon-transcript"  # Specific features
```

**Default features:** `opus`, `console`, `addon-acme`, `addon-transcript`, `addon-archive`.
See `Cargo.toml` `[features]` for the full list.

### Running Locally

Create a minimal `config.toml`:

```toml
http_addr = "0.0.0.0:8080"
log_level = "debug"
database_url = "sqlite://rustpbx.sqlite3"

[console]
base_path = "/console"
allow_registration = true

[proxy]
modules = ["acl", "auth", "presence", "registrar", "call"]
addr = "0.0.0.0"
udp_port = 5060
ws_handler = "/ws"
acl_rules = ["allow all"]

[[proxy.user_backends]]
type = "memory"
users = [
    { username = "1001", password = "test1001" },
    { username = "1002", password = "test1002" },
]
```

```bash
# Run the server
./target/release/rustpbx --conf config.toml

# Create a console admin user
./target/release/rustpbx --conf config.toml \
    --super-username admin --super-password admin123 --super-email admin@localhost
```

Listens on: HTTP `:8080` (console at `/console`), SIP/UDP `:5060`, WebSocket at `/ws`.

### Docker

```bash
docker pull ghcr.io/restsend/rustpbx:latest

# On Windows Git Bash, prefix with MSYS_NO_PATHCONV=1
docker run -d -p 8080:8080 -p 5060:5060/udp \
    -v $(pwd)/config.toml:/app/config.toml \
    ghcr.io/restsend/rustpbx:latest --conf /app/config.toml
```

### Hot Reload

No built-in hot reload. Use `cargo watch` for automatic rebuilds:

```bash
cargo install cargo-watch
cargo watch -x 'run -- --conf config.toml'
```

---

## 4. Running Tests

### Unit Tests

```bash
cargo test                                  # All tests
cargo test --lib proxy::tests              # Proxy module tests
cargo test --lib media::recorder_tests     # Recorder tests
cargo test test_acl_allow_all              # Single test by name
cargo test -- --nocapture                  # With stdout visible
```

**Key test locations:**
- `src/proxy/tests/` -- ACL, auth, registrar, B2BUA flow, CDR, presence, queue
- `src/media/*_tests.rs` -- Media track, recorder, file track tests
- `src/proxy/proxy_call/callsession_tests.rs` -- B2BUA session tests

### Integration Tests

```bash
cargo test --test audio_feature_test
cargo test --test ringback_mode_test
```

### Python End-to-End Tests

Require a running RustPBX instance:

```bash
pip install -r tests/requirements.txt
python -m pytest tests/test_L0_smoke.py    # No server needed
python -m pytest tests/test_L2_api.py      # API tests (server required)
python -m pytest tests/test_L3_sip.py      # SIP protocol tests
python tests/sip_test_call.py              # Manual SIP test call
```

---

## 5. Debugging SIP Calls

### SipFlow Traces

Enable per-call SIP message capture:

```toml
[sipflow]
type = "local"
root = "./config/sipflow"
subdirs = "daily"
```

View in the console UI at `/console/sipflow` or with the standalone binary:
`cargo run --bin sipflow -- --path ./config/sipflow/2026-02-24/`

### Log Filtering

```bash
RUST_LOG=rustpbx::proxy::call=trace,rustpbx::media=debug ./target/release/rustpbx --conf config.toml
```

Key log targets: `rustpbx::proxy::call` (routing), `rustpbx::proxy::auth` (authentication),
`rustpbx::proxy::registrar` (registration), `rustpbx::proxy::proxy_call` (B2BUA session),
`rustpbx::media` (tracks/recording), `rsipstack` (transport/transaction).

### Common Issues

| Symptom | Cause | Fix |
|---------|-------|-----|
| 403 on INVITE | ACL blocking source IP | Check `acl_rules`, source IP in logs |
| 407 auth loop | Wrong credentials or realm mismatch | Check `realms` config, user backend passwords |
| No audio / one-way audio | NAT/firewall blocking RTP | Set `external_ip`, open RTP port range |
| ACK timeout after 200 OK | NatInspector corrupting Contact | Check Record-Route presence, NAT config |
| WebRTC no media | ICE/DTLS failure | Check browser console, `ice_servers` config |
| "not in same realm" | SIP URI host missing from realms | Add host to `realms = [...]` |

### Packet Capture

```bash
sudo sngrep -d eth0 port 5060                  # SIP traffic
# Wireshark: sip || rtp (display filter)
```

---

## 6. Key Code Patterns

### ProxyModule Trait

Every SIP request passes through the module pipeline. Modules return `ProxyAction::Continue`
or `ProxyAction::Abort`:

```rust
#[async_trait]
pub trait ProxyModule: Send + Sync {
    fn name(&self) -> &str;
    fn allow_methods(&self) -> Vec<rsip::Method> { vec![] }
    async fn on_start(&mut self) -> Result<()>;
    async fn on_stop(&self) -> Result<()>;
    async fn on_transaction_begin(
        &self, token: CancellationToken, tx: &mut Transaction, cookie: TransactionCookie,
    ) -> Result<ProxyAction> { Ok(ProxyAction::Continue) }
    async fn on_transaction_end(&self, tx: &mut Transaction) -> Result<()> { Ok(()) }
}
```

### SessionAction for Call Control

The B2BUA session state machine is driven by `SessionAction` variants sent via `mpsc`:

```rust
pub enum SessionAction {
    AcceptCall { callee, sdp, dialog_id },
    TransferTarget(String),
    ProvideEarlyMedia(String),
    StartRinging { ringback, passthrough },
    PlayPrompt { audio_file, send_progress, await_completion },
    Hangup { reason, code, initiator },
    HandleReInvite(method, sdp),
    RefreshSession, MuteTrack(id), UnmuteTrack(id),
}
```

### MediaBridge and MediaPeer

`MediaPeer` wraps a set of `Track` objects for one call leg. `MediaBridge` connects
two peers, forwarding and transcoding media bidirectionally:

```rust
pub trait MediaPeer: Send + Sync {
    async fn update_track(&self, track: Box<dyn Track>, play_id: Option<String>);
    async fn get_tracks(&self) -> Vec<Arc<AsyncMutex<Box<dyn Track>>>>;
    async fn suppress_forwarding(&self, track_id: &str);
    async fn resume_forwarding(&self, track_id: &str);
    async fn serve(&self) -> Result<()>;
    fn stop(&self);
}

pub trait Track: Send + Sync {
    fn id(&self) -> &str;
    async fn handshake(&self, remote_offer: String) -> Result<String>;
    async fn local_description(&self) -> Result<String>;
    async fn set_remote_description(&self, remote: &str) -> Result<()>;
    async fn stop(&self);
}
```

Implementations: `RtpTrack` (plain RTP), `RtcTrack` (WebRTC), `FileTrack` (playback).

### Console Handler Pattern

Handlers use axum `State` + `AuthRequired` extractor, render templates with `state.render()`,
and expose a `urls()` function for route registration:

```rust
async fn page_extensions(
    State(state): State<Arc<ConsoleState>>, AuthRequired(_): AuthRequired,
) -> Response {
    state.render("console/extensions.html", json!({ "nav_active": "extensions" }))
}

pub fn urls() -> Router<Arc<ConsoleState>> {
    Router::new()
        .route("/extensions", get(page_extensions).put(create_extension))
        .route("/extensions/{id}", get(page_extension_detail).patch(update_extension))
}
```

Routes are merged in `src/console/handlers/mod.rs` via `.merge(extension::urls())`.

---

## 7. Adding a New SIP Module

**Step 1:** Create `src/proxy/rate_limiter.rs` implementing `ProxyModule`:

```rust
use super::{ProxyAction, ProxyModule, server::SipServerRef};
use crate::{call::TransactionCookie, config::ProxyConfig};
use anyhow::Result;
use async_trait::async_trait;
use rsipstack::transaction::transaction::Transaction;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub struct RateLimiterModule { server: SipServerRef, config: Arc<ProxyConfig> }

#[async_trait]
impl ProxyModule for RateLimiterModule {
    fn name(&self) -> &str { "rate_limiter" }
    fn allow_methods(&self) -> Vec<rsip::Method> { vec![rsip::Method::Invite] }
    async fn on_start(&mut self) -> Result<()> { Ok(()) }
    async fn on_stop(&self) -> Result<()> { Ok(()) }
    async fn on_transaction_begin(
        &self, _token: CancellationToken, tx: &mut Transaction, cookie: TransactionCookie,
    ) -> Result<ProxyAction> {
        // Rate limiting logic -- return Abort to reject
        Ok(ProxyAction::Continue)
    }
}
```

**Step 2:** Add `pub mod rate_limiter;` in `src/proxy/mod.rs`.

**Step 3:** In `src/app.rs`, add a match arm in the module creation loop:

```rust
"rate_limiter" => Box::new(rate_limiter::RateLimiterModule::new(server_ref, proxy_config)?)
```

**Step 4:** Enable in config: `modules = ["acl", "rate_limiter", "auth", "registrar", "call"]`

Pipeline order matters -- place rate limiting after ACL but before auth.

---

## 8. Adding a Console UI Page

**Step 1:** Create `src/console/handlers/reports.rs`:

```rust
use crate::console::{ConsoleState, middleware::AuthRequired};
use axum::{Router, extract::State, response::Response, routing::get};
use serde_json::json;
use std::sync::Arc;

async fn page_reports(
    State(state): State<Arc<ConsoleState>>, AuthRequired(_): AuthRequired,
) -> Response {
    state.render("console/reports.html", json!({ "nav_active": "reports" }))
}

pub fn urls() -> Router<Arc<ConsoleState>> {
    Router::new().route("/reports", get(page_reports))
}
```

**Step 2:** In `src/console/handlers/mod.rs`, add `pub mod reports;` and merge in `router()`:
`.merge(reports::urls())`

**Step 3:** Create `templates/console/reports.html`:

```html
{% extends "console/layout.html" %}
{% block content %}
<div x-data="{ items: [] }" x-init="fetch('{{ base_path }}/reports/data').then(r => r.json()).then(d => items = d.items)">
    <h1 class="text-2xl font-bold mb-6">Reports</h1>
    <template x-for="item in items" :key="item.id">
        <div class="border-b py-3" x-text="item.name"></div>
    </template>
</div>
{% endblock %}
```

**Step 4:** Add a sidebar link in `templates/console/layout.html`.

---

## 9. Database Migrations

RustPBX uses SeaORM with idempotent migrations that run automatically at startup.

### Adding a Column

**1.** Create `src/models/add_report_status.rs`:

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !manager.has_column("rustpbx_call_records", "report_status").await? {
            manager.alter_table(
                Table::alter()
                    .table(crate::models::call_record::Entity)
                    .add_column(ColumnDef::new(Alias::new("report_status")).string().null())
                    .to_owned(),
            ).await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.alter_table(
            Table::alter()
                .table(crate::models::call_record::Entity)
                .drop_column(Alias::new("report_status"))
                .to_owned(),
        ).await
    }
}
```

**2.** Declare in `src/models/mod.rs`: `pub mod add_report_status;`

**3.** Append to `src/models/migration.rs` `migrations()` vec:
`Box::new(super::add_report_status::Migration)`

**4.** Add the field to the entity model struct: `pub report_status: Option<String>`

### Rules

- **Always append** migrations at the end of the vec -- never reorder.
- **Always check** `has_column()` / `has_table()` before altering (idempotent).
- Support both SQLite and MySQL -- avoid database-specific SQL.
- For new tables, see `src/models/extension.rs` as a complete example.

---

## 10. Code Style

### Error Handling

Use `anyhow::Result` for application code. Chain context with `.with_context()`:

```rust
let content = std::fs::read_to_string(path)
    .with_context(|| format!("failed to read config: {}", path))?;
```

### Logging

Use `tracing` macros. Levels: `error!` (unrecoverable), `warn!` (degraded),
`info!` (lifecycle events), `debug!` (operational detail), `trace!` (packet-level).
Use structured fields: `info!(call_id = %id, "call established");`

### Async Patterns

- **Runtime:** tokio multi-threaded
- **Shutdown:** `CancellationToken` from `tokio_util` (parent/child propagation)
- **Shared state:** `Arc<Mutex<T>>` (prefer `tokio::sync::Mutex` for async)
- **Communication:** `tokio::sync::mpsc` channels between tasks
- **Concurrency:** `tokio::select!` for multiplexing futures

### Naming

- Files: `snake_case.rs`; Structs: `PascalCase`; Functions: `snake_case`; Constants: `SCREAMING_SNAKE_CASE`
- DB tables: `rustpbx_` prefix (e.g., `rustpbx_extensions`)
- SIP types preserve standard terminology (`Transaction`, `Dialog`, `Uri`)

### Feature Gates

```rust
#[cfg(feature = "console")]
pub mod console;
```

### Config Structs

Use `serde::Deserialize` with `#[serde(default = "...")]` for defaults:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct MyConfig {
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}
fn default_timeout() -> u64 { 30 }
```

### Tests

Use `#[cfg(test)]` modules alongside source. Dev dependencies: `tempfile` (temp dirs),
`portpicker` (unused ports), `mockall` (mocking), `tokio-test`. See
`src/proxy/tests/common.rs` for shared test utilities.

---

## Quick Reference

```bash
cargo build --release                          # Build
./target/release/rustpbx --conf config.toml    # Run
cargo test                                     # All tests
cargo test -- --nocapture                      # Tests with output
cargo clippy --all-targets --all-features      # Lint
cargo fmt --all                                # Format
./target/release/rustpbx --version             # Version
```
