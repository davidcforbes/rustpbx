# Active-Call Integration Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Port miuda-ai/active-call's AI voice agent modules into our RustPBX fork, creating a unified PBX + AI Voice Agent platform.

**Architecture:** Add active-call's real-time AI pipeline (VAD, streaming ASR, LLM, TTS, playbook engine, offline ONNX models) as new modules alongside our existing PBX core (proxy, console, voicemail, sipflow). Resolve naming conflicts by keeping our PBX `Track`/`MediaStream` intact and porting active-call's media pipeline under namespaced submodules. The merged binary serves both roles: SIP proxy/PBX AND AI voice agent.

**Tech Stack:** Rust 2024 edition, rsipstack 0.4.x, rustrtc 0.3.22, ONNX Runtime (ort), nnnoiseless, Silero VAD, SenseVoice ASR, Supertonic TTS, axum, tokio

**Source:** `https://github.com/miuda-ai/active-call.git` cloned to `/tmp/active-call`

---

## Phase 1: Dependencies & Configuration (Cargo.toml)

### Task 1.1: Update Existing Dependency Versions

**Files:**
- Modify: `Cargo.toml`

**Step 1: Bump versions in Cargo.toml**

Update these existing dependencies to match active-call's newer versions:
- `rustrtc`: `"0.3.21"` → `"0.3.22"`
- `uuid`: `"1.19.0"` → `"1.20.0"` (add `features = ["v4"]`)
- `hound`: `"3.5"` → `"3.5.1"`

**Step 2: Verify compilation**

```bash
cargo check 2>&1 | head -30
```

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump rustrtc, uuid, hound versions for active-call compat"
```

---

### Task 1.2: Add New Dependencies for AI Pipeline

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add mandatory new dependencies**

Add to `[dependencies]` section:
```toml
# AI Voice Pipeline - Audio Processing
nnnoiseless = "0.5.2"
realfft = "3.3"
rustfft = "6.2"
num-complex = "0.4"
rmp3 = "0.3"

# AI Voice Pipeline - Networking & System
get_if_addrs = "0.5.3"
socket2 = { version = "0.6", features = ["all"] }
num_cpus = "1.16"

# AI Voice Pipeline - Text Processing
unicode-normalization = "0.1"
serde_yaml = "0.9.33"

# AI Voice Pipeline - Synthesis/Transcription client support
object_store = { version = "0.13.1", features = ["aws", "gcp", "azure"] }
```

**Step 2: Add optional (feature-gated) dependencies**

```toml
# Offline AI Models (SenseVoice ASR + Supertonic TTS)
symphonia = { version = "0.5", optional = true, features = ["all"] }
knf-rs-sys = { version = "0.3.2", optional = true }
ort = { version = "2.0.0-rc.11", optional = true, features = ["std", "ndarray", "load-dynamic"] }
hf-hub = { version = "0.4.3", optional = true, features = ["tokio"] }
ndarray = { version = "0.16.1", optional = true }
```

**Step 3: Add new feature flags**

```toml
[features]
default = ["opus", "console", "addon-acme", "addon-transcript", "addon-archive", "voice-agent"]
voice-agent = []
offline = ["dep:symphonia", "dep:knf-rs-sys", "dep:ort", "dep:hf-hub", "dep:ndarray"]
```

**Step 4: Verify compilation**

```bash
cargo check 2>&1 | head -30
```

**Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add active-call AI pipeline dependencies"
```

---

## Phase 2: Port Standalone New Modules (No Conflicts)

These modules are entirely new — they don't exist in our codebase at all.

### Task 2.1: Port Event System

**Files:**
- Create: `src/event.rs`
- Modify: `src/lib.rs`

**Step 1: Copy event.rs from active-call**

```bash
cp /tmp/active-call/src/event.rs src/event.rs
```

**Step 2: Add module declaration to lib.rs**

Add `pub mod event;` to `src/lib.rs`.

**Step 3: Fix imports**

Update `use crate::media::PcmBuf` references — our media module may not export `PcmBuf`. Check if this type exists; if not, add it to `src/media/mod.rs` or adapt the import.

**Step 4: Verify compilation**

```bash
cargo check 2>&1 | grep "event" | head -10
```

**Step 5: Commit**

```bash
git add src/event.rs src/lib.rs
git commit -m "feat: add session event system from active-call"
```

---

### Task 2.2: Port Network Utilities

**Files:**
- Create: `src/net_tool.rs`
- Modify: `src/lib.rs`

**Step 1: Copy net_tool.rs**

```bash
cp /tmp/active-call/src/net_tool.rs src/net_tool.rs
```

**Step 2: Add module declaration to lib.rs**

Add `pub mod net_tool;` to `src/lib.rs`.

**Step 3: Fix imports** (should be minimal — uses `get_if_addrs`, `anyhow`)

**Step 4: Verify compilation, commit**

```bash
cargo check 2>&1 | grep "net_tool" | head -5
git add src/net_tool.rs src/lib.rs
git commit -m "feat: add network utility functions from active-call"
```

---

### Task 2.3: Port Synthesis Module (TTS Providers)

**Files:**
- Create: `src/synthesis/` (entire directory)
- Modify: `src/lib.rs`

**Step 1: Copy entire synthesis directory**

```bash
cp -r /tmp/active-call/src/synthesis src/synthesis
```

**Step 2: Add module declaration to lib.rs**

Add `pub mod synthesis;` to `src/lib.rs`.

**Step 3: Fix imports**

The synthesis module references:
- `crate::media::*` — May need to add `PcmBuf`, `Samples`, `AudioFrame` types to our media module
- `crate::offline::SupertonicTts` — Will be available after Task 2.6

For now, wrap offline references with `#[cfg(feature = "offline")]`.

**Step 4: Iteratively fix compilation errors**

```bash
cargo check 2>&1 | grep "error" | head -20
```

Fix each import mismatch. Common fixes:
- Add missing type aliases to `src/media/mod.rs`
- Gate offline features with `#[cfg(feature = "offline")]`

**Step 5: Commit**

```bash
git add src/synthesis/ src/lib.rs
git commit -m "feat: add TTS synthesis providers (Aliyun, Deepgram, Tencent, Supertonic)"
```

---

### Task 2.4: Port Transcription Module (Streaming ASR Providers)

**Files:**
- Create: `src/transcription/` (entire directory — NOT the same as our `src/addons/transcript/`)
- Modify: `src/lib.rs`

**Step 1: Copy entire transcription directory**

```bash
cp -r /tmp/active-call/src/transcription src/transcription
```

**Note:** Our existing `src/addons/transcript/` is a POST-CALL transcription hook. This new `src/transcription/` is REAL-TIME streaming ASR. They are complementary, not redundant. Keep both.

**Step 2: Add module declaration to lib.rs**

Add `pub mod transcription;` to `src/lib.rs`.

**Step 3: Fix imports and compile**

References `crate::media::*` types and `crate::offline::SensevoiceEncoder`.

**Step 4: Commit**

```bash
git add src/transcription/ src/lib.rs
git commit -m "feat: add streaming ASR transcription (Aliyun, Tencent, SenseVoice)"
```

---

### Task 2.5: Port Playbook Engine

**Files:**
- Create: `src/playbook/` (entire directory)
- Modify: `src/lib.rs`

**Step 1: Copy entire playbook directory**

```bash
cp -r /tmp/active-call/src/playbook src/playbook
```

**Step 2: Add module declaration to lib.rs**

Add `pub mod playbook;` to `src/lib.rs`.

**Step 3: Fix imports**

The playbook module references:
- `crate::call::Command` — This is active-call's `Command` enum, not our call module. Will need to add this type.
- `crate::event::SessionEvent`
- `crate::synthesis::*`
- `crate::transcription::*`
- `crate::media::*`
- `minijinja` (already in our deps for console)

**Step 4: Iteratively fix compilation, commit**

```bash
git add src/playbook/ src/lib.rs
git commit -m "feat: add AI playbook engine with LLM integration"
```

---

### Task 2.6: Port Offline AI Models

**Files:**
- Create: `src/offline/` (entire directory)
- Copy: `src/media/vad/silero_weights.bin` (binary model file)
- Modify: `src/lib.rs`

**Step 1: Copy offline directory**

```bash
cp -r /tmp/active-call/src/offline src/offline
```

**Step 2: Add feature-gated module to lib.rs**

```rust
#[cfg(feature = "offline")]
pub mod offline;
```

**Step 3: Verify ONNX Runtime dependency compiles**

The `ort` crate needs ONNX Runtime. With `load-dynamic` feature, it loads at runtime rather than linking.

**Step 4: Commit**

```bash
git add src/offline/ src/lib.rs
git commit -m "feat: add offline AI models (SenseVoice ASR, Supertonic TTS)"
```

---

### Task 2.7: Port User Agent Module

**Files:**
- Create: `src/useragent/` (entire directory)
- Modify: `src/lib.rs`

**Step 1: Copy useragent directory**

```bash
cp -r /tmp/active-call/src/useragent src/useragent
```

**Step 2: Add module declaration to lib.rs**

Add `pub mod useragent;` to `src/lib.rs`.

**Step 3: Fix imports**

References `crate::app::AppState`, `crate::call::*`, `crate::playbook::*`, `crate::config::*`.

**Step 4: Commit**

```bash
git add src/useragent/ src/lib.rs
git commit -m "feat: add SIP user agent for AI voice calls"
```

---

## Phase 3: Merge Overlapping Modules

### Task 3.1: Extend Media Module with AI Processing Pipeline

This is the most complex merge. Our existing media module has: `recorder.rs`, `transcoder.rs`, `negotiate.rs`, `audio_source.rs`, `call_quality.rs`, `wav_writer.rs`. Active-call adds ~20 new files.

**Files:**
- Create: `src/media/vad/` (directory with 4 files + binary weights)
- Create: `src/media/track/` (directory with 7 files)
- Create: `src/media/ambiance.rs`
- Create: `src/media/asr_processor.rs`
- Create: `src/media/cache.rs`
- Create: `src/media/denoiser.rs`
- Create: `src/media/dtmf.rs`
- Create: `src/media/engine.rs`
- Create: `src/media/inactivity.rs`
- Create: `src/media/loader.rs`
- Create: `src/media/processor.rs`
- Create: `src/media/agent_stream.rs` (renamed from active-call's `stream.rs` to avoid conflict)
- Create: `src/media/realtime_processor.rs`
- Create: `src/media/volume_control.rs`
- Modify: `src/media/mod.rs`

**Step 1: Copy new files that don't conflict**

```bash
# New directories
cp -r /tmp/active-call/src/media/vad src/media/vad
cp -r /tmp/active-call/src/media/track src/media/track

# New standalone files
cp /tmp/active-call/src/media/ambiance.rs src/media/ambiance.rs
cp /tmp/active-call/src/media/asr_processor.rs src/media/asr_processor.rs
cp /tmp/active-call/src/media/cache.rs src/media/cache.rs
cp /tmp/active-call/src/media/denoiser.rs src/media/denoiser.rs
cp /tmp/active-call/src/media/dtmf.rs src/media/dtmf.rs
cp /tmp/active-call/src/media/engine.rs src/media/engine.rs
cp /tmp/active-call/src/media/inactivity.rs src/media/inactivity.rs
cp /tmp/active-call/src/media/loader.rs src/media/loader.rs
cp /tmp/active-call/src/media/processor.rs src/media/processor.rs
cp /tmp/active-call/src/media/realtime_processor.rs src/media/realtime_processor.rs
cp /tmp/active-call/src/media/volume_control.rs src/media/volume_control.rs

# Rename stream.rs to avoid conflict with our MediaStream
cp /tmp/active-call/src/media/stream.rs src/media/agent_stream.rs
```

**Step 2: Add shared types to media/mod.rs**

Add these types that active-call modules expect (from active-call's `media/mod.rs`):

```rust
// Types needed by AI voice pipeline
pub type TrackId = String;
pub type PayloadBuf = Vec<u8>;

pub enum Samples {
    PCM(Vec<i16>),
}

pub struct SourcePacket {
    pub track_id: TrackId,
    pub samples: Samples,
    pub codec: audio_codec::CodecType,
    pub ssrc: u32,
    pub timestamp: u32,
    pub payload: PayloadBuf,
}

pub struct AudioFrame {
    pub track_id: TrackId,
    pub samples: Vec<i16>,
    pub sample_rate: u32,
    pub timestamp: u64,
}

pub type PcmBuf = Vec<i16>;
```

**Step 3: Add module declarations to media/mod.rs**

```rust
// AI Voice Agent media processing (from active-call)
pub mod ambiance;
pub mod asr_processor;
pub mod cache;
pub mod denoiser;
pub mod dtmf;
pub mod engine;
pub mod inactivity;
pub mod loader;
pub mod processor;
pub mod agent_stream;
pub mod realtime_processor;
pub mod volume_control;
pub mod track;
pub mod vad;
```

**Step 4: Handle RecorderOption conflict**

Our `RecorderOption` has `input_gain`/`output_gain` fields. Active-call's has `format` field. Create unified version:

```rust
pub struct RecorderOption {
    pub recorder_file: String,
    pub samplerate: u32,
    pub ptime: u32,
    pub input_gain: f32,               // from RustPBX
    pub output_gain: f32,              // from RustPBX
    pub format: Option<RecorderFormat>, // from active-call
}
```

**Step 5: Fix active-call references to `crate::media::stream::*`**

In `src/media/agent_stream.rs`, update:
- `mod stream` references to `mod agent_stream`
- Internal `use super::*` should work since it's in the same media module

**Step 6: Fix Track trait conflict**

Active-call's `Track` trait in `src/media/track/mod.rs` conflicts with our `Track` in `src/media/mod.rs`. They serve different purposes:
- Ours: WebRTC/RTP peer connection management
- Active-call's: Audio processing pipeline with processor chains

Resolution: Rename our existing `Track` → `PeerTrack` across the codebase (it's used in proxy_call code), OR prefix active-call's Track. Since active-call has more files referencing its `Track`, rename ours.

Search all usages of our `Track` trait:
```bash
grep -rn "dyn Track" src/media/ src/proxy/
```

Rename: `Track` → `PeerTrack`, `MediaStream` stays as-is (our proxy version). Active-call's `MediaStream` becomes `AgentMediaStream` in `agent_stream.rs`.

**Step 7: Iteratively compile and fix**

```bash
cargo check 2>&1 | grep "error" | head -20
```

**Step 8: Commit**

```bash
git add src/media/
git commit -m "feat: add AI media pipeline (VAD, denoiser, DTMF, processor chain, tracks)"
```

---

### Task 3.2: Port Active Call Module (AI Call Lifecycle)

**Files:**
- Create: `src/call/active_call.rs`
- Create: `src/call/active_sip.rs` (renamed from active-call's `call/sip.rs`)
- Create: `src/call/command.rs` (extracted from active-call's `call/mod.rs`)
- Modify: `src/call/mod.rs`

**Step 1: Copy active-call's call module files**

```bash
cp /tmp/active-call/src/call/active_call.rs src/call/active_call.rs
cp /tmp/active-call/src/call/sip.rs src/call/active_sip.rs
```

**Step 2: Extract Command enum from active-call's call/mod.rs**

Create `src/call/command.rs` with the `Command` enum and `RoutingState` from active-call's `call/mod.rs`.

**Step 3: Add module declarations to our call/mod.rs**

```rust
// AI Voice Agent call management (from active-call)
pub mod active_call;
pub mod active_sip;
pub mod command;

pub use command::Command as AgentCommand;
```

**Step 4: Fix imports in active_call.rs and active_sip.rs**

- Replace `super::Command` → `super::command::Command`
- Replace `crate::app::AppState` references — our AppState is different. Need to create an adapter or extend ours.

**Step 5: Compile and fix**

**Step 6: Commit**

```bash
git add src/call/
git commit -m "feat: add AI active call lifecycle management"
```

---

### Task 3.3: Merge Configuration

**Files:**
- Modify: `src/config.rs`

**Step 1: Add AI voice agent config sections**

Add these structs to our `src/config.rs` (from active-call's config.rs):

```rust
/// Configuration for AI voice agent functionality
#[derive(Debug, Clone, Deserialize, Default)]
pub struct VoiceAgentConfig {
    /// HTTP address for voice agent API (default: disabled)
    pub http_addr: Option<String>,
    /// SIP bind address for voice agent UA
    pub sip_addr: Option<String>,
    /// SIP UDP port (default: 25060)
    pub sip_port: Option<u16>,
    /// Invite handler configuration
    pub handler: Option<InviteHandlerConfig>,
    /// Recording policy for AI calls
    pub recording: Option<RecordingPolicy>,
    /// SIP registrations (register as extension on other PBXes)
    pub register_users: Vec<RegisterOption>,
    /// ICE servers for WebRTC
    pub ice_servers: Vec<IceServer>,
    /// URI rewrite rules
    pub rewrites: Vec<RewriteRule>,
    /// Codec preferences
    pub codecs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum InviteHandlerConfig {
    #[serde(rename = "webhook")]
    Webhook { url: Option<String>, urls: Option<Vec<String>>, method: Option<String> },
    #[serde(rename = "playbook")]
    Playbook { rules: Vec<PlaybookRule>, default: Option<String> },
}
```

**Step 2: Add `voice_agent` field to top-level Config**

```rust
pub struct Config {
    // ... existing fields ...
    pub voice_agent: Option<VoiceAgentConfig>,
}
```

**Step 3: Commit**

```bash
git add src/config.rs
git commit -m "feat: add voice agent configuration sections"
```

---

## Phase 4: Integration & Wiring

### Task 4.1: Extend AppState with Voice Agent

**Files:**
- Modify: `src/app.rs`

**Step 1: Add voice agent fields to AppStateInner**

```rust
pub struct AppStateInner {
    // ... existing fields ...

    /// Stream engine for AI voice processing (VAD, ASR, TTS registry)
    #[cfg(feature = "voice-agent")]
    pub stream_engine: Option<Arc<crate::media::engine::StreamEngine>>,

    /// Active AI voice calls
    #[cfg(feature = "voice-agent")]
    pub voice_calls: Arc<tokio::sync::RwLock<HashMap<String, crate::call::active_call::ActiveCallRef>>>,
}
```

**Step 2: Extend AppStateBuilder to initialize voice agent**

In `build()`, after existing initialization:

```rust
#[cfg(feature = "voice-agent")]
let stream_engine = if config.voice_agent.is_some() {
    let mut engine = StreamEngine::new();
    // Register built-in providers
    engine.register_vad("silero", ...);
    Some(Arc::new(engine))
} else {
    None
};
```

**Step 3: Commit**

```bash
git add src/app.rs
git commit -m "feat: extend AppState with voice agent stream engine"
```

---

### Task 4.2: Add Voice Agent HTTP Routes

**Files:**
- Create: `src/handler/voice_agent.rs`
- Modify: `src/handler/mod.rs`

**Step 1: Create voice agent route handler**

Adapt active-call's `handler/handler.rs` routes to integrate with our axum router:

```rust
// src/handler/voice_agent.rs
pub fn voice_agent_router() -> Router<AppState> {
    Router::new()
        .route("/voice/call", get(ws_call_handler))
        .route("/voice/call/webrtc", get(webrtc_call_handler))
        .route("/voice/call/sip", get(sip_call_handler))
        .route("/voice/list", get(list_active_calls))
        .route("/voice/kill/:id", get(kill_call))
        .route("/voice/playbooks", get(list_playbooks))
        .route("/voice/playbooks/:name", get(get_playbook).post(save_playbook))
}
```

**Step 2: Add route to main handler**

In `src/handler/mod.rs`, merge the voice agent router.

**Step 3: Commit**

```bash
git add src/handler/
git commit -m "feat: add voice agent HTTP/WebSocket routes"
```

---

### Task 4.3: Port Playbook Examples and Binary Assets

**Files:**
- Create: `config/playbook/` (directory)
- Copy: Binary model weights

**Step 1: Copy playbook examples**

```bash
mkdir -p config/playbook
cp /tmp/active-call/config/playbook/*.md config/playbook/
```

**Step 2: Copy VAD model weights**

```bash
cp /tmp/active-call/src/media/vad/silero_weights.bin src/media/vad/silero_weights.bin
```

**Step 3: Copy office ambiance audio**

```bash
mkdir -p config
cp /tmp/active-call/config/office.wav config/office.wav
```

**Step 4: Copy WebRTC client page**

```bash
cp /tmp/active-call/static/index.html static/voice_agent.html
```

**Step 5: Commit**

```bash
git add config/playbook/ src/media/vad/silero_weights.bin config/office.wav static/voice_agent.html
git commit -m "feat: add playbook examples, VAD model, and voice agent assets"
```

---

## Phase 5: Compile, Fix, and Stabilize

### Task 5.1: Resolve All Compilation Errors

**Step 1: Full cargo check**

```bash
cargo check 2>&1 | tee /tmp/build-errors.txt
```

**Step 2: Categorize errors**

Common error categories and fixes:
- **Missing type imports**: Add `use crate::media::*` or create type aliases
- **`crate::app::AppState` mismatch**: Active-call expects its AppState. Create a trait or adapter.
- **`rand` version conflicts**: Active-call uses `rand 0.8.5`, we use `0.10.0`. Port active-call's rand usage to 0.10 API (main change: `Rng::gen_range(a..b)` syntax).
- **`minijinja` usage**: Active-call uses it directly for playbook templates. Our fork gates it behind `console` feature. Either make it always available or gate playbook behind `console` too.
- **Feature gates**: Wrap offline-dependent code with `#[cfg(feature = "offline")]`

**Step 3: Fix errors iteratively**

Work through build errors one module at a time:
1. `event.rs` (fewest dependencies)
2. `net_tool.rs` (standalone)
3. `media/processor.rs` → `media/vad/` → `media/denoiser.rs` → remaining media
4. `synthesis/` (depends on media)
5. `transcription/` (depends on media)
6. `call/command.rs` → `call/active_call.rs` → `call/active_sip.rs`
7. `playbook/` (depends on call, synthesis, transcription)
8. `useragent/` (depends on playbook, call)
9. `handler/voice_agent.rs` (depends on everything)

**Step 4: Commit after each module compiles**

### Task 5.2: Run Existing Tests

**Step 1: Ensure existing PBX tests still pass**

```bash
cargo test 2>&1 | tee /tmp/test-results.txt
```

**Step 2: Fix any regressions from the merge**

The Track trait rename (→ PeerTrack) will break proxy_call tests. Fix all references.

**Step 3: Commit**

```bash
git add -A
git commit -m "fix: resolve compilation errors and test regressions from merge"
```

---

## Phase 6: Cleanup Redundant Code

### Task 6.1: Audit for Redundancy

Review these areas where active-call provides superior replacements:

| Our Module | Active-Call Replacement | Action |
|------------|------------------------|--------|
| `media/negotiate.rs` | `media/negotiate.rs` (with hold detection, IPv6 stripping) | Merge extras into ours |
| `addons/transcript/hook.rs` | Can use `transcription/sensevoice.rs` for real-time ASR | Keep both — different use cases |
| `media/recorder.rs` | `media/recorder.rs` (with format selection) | Merge `RecorderFormat` enum into ours |
| `net_tool.rs` (new) vs scattered NAT code | Consolidated utility | Use net_tool.rs, remove duplicates |

### Task 6.2: Remove Dead Code

After integration, check for:
```bash
cargo check 2>&1 | grep "warning.*dead_code\|warning.*unused"
```

Remove or prefix with `_` as appropriate.

### Task 6.3: Final Commit

```bash
git add -A
git commit -m "refactor: remove redundant code after active-call integration"
```

---

## Module Map After Merge

```
src/
├── lib.rs                  (MODIFIED: +7 module declarations)
├── app.rs                  (MODIFIED: +StreamEngine, +voice_calls)
├── config.rs               (MODIFIED: +VoiceAgentConfig)
├── event.rs                (NEW: session event system)
├── net_tool.rs             (NEW: network utilities)
│
├── call/                   (EXTENDED)
│   ├── mod.rs              (MODIFIED: +3 module declarations)
│   ├── sip.rs              (KEPT: our PBX SIP abstractions)
│   ├── user.rs             (KEPT)
│   ├── cookie.rs           (KEPT)
│   ├── policy.rs           (KEPT)
│   ├── queue_config.rs     (KEPT)
│   ├── active_call.rs      (NEW: AI voice call lifecycle)
│   ├── active_sip.rs       (NEW: AI call SIP dialog handling)
│   └── command.rs          (NEW: AI call command enum)
│
├── media/                  (EXTENDED HEAVILY)
│   ├── mod.rs              (MODIFIED: +types, +14 module decls)
│   ├── recorder.rs         (MODIFIED: unified RecorderOption)
│   ├── transcoder.rs       (KEPT)
│   ├── negotiate.rs        (MERGED: +hold detection, +IPv6)
│   ├── audio_source.rs     (KEPT)
│   ├── call_quality.rs     (KEPT)
│   ├── wav_writer.rs       (KEPT)
│   ├── ambiance.rs         (NEW: background audio)
│   ├── asr_processor.rs    (NEW: ASR integration)
│   ├── cache.rs            (NEW: media caching)
│   ├── denoiser.rs         (NEW: nnnoiseless)
│   ├── dtmf.rs             (NEW: DTMF detection)
│   ├── engine.rs           (NEW: StreamEngine registry)
│   ├── inactivity.rs       (NEW: timeout processor)
│   ├── loader.rs           (NEW: audio file loading)
│   ├── processor.rs        (NEW: ProcessorChain)
│   ├── agent_stream.rs     (NEW: AI call media stream)
│   ├── realtime_processor.rs (NEW: OpenAI Realtime)
│   ├── volume_control.rs   (NEW: volume ducking)
│   ├── track/              (NEW: media track types)
│   │   ├── mod.rs
│   │   ├── file.rs
│   │   ├── media_pass.rs
│   │   ├── rtc.rs
│   │   ├── tts.rs
│   │   ├── track_codec.rs
│   │   └── websocket.rs
│   └── vad/                (NEW: voice activity detection)
│       ├── mod.rs
│       ├── tiny_silero.rs
│       ├── simd.rs
│       ├── utils.rs
│       └── silero_weights.bin
│
├── synthesis/              (NEW: TTS providers)
│   ├── mod.rs
│   ├── aliyun.rs
│   ├── deepgram.rs
│   ├── supertonic.rs
│   ├── tencent_cloud.rs
│   └── tencent_cloud_basic.rs
│
├── transcription/          (NEW: streaming ASR providers)
│   ├── mod.rs
│   ├── aliyun.rs
│   ├── sensevoice.rs
│   └── tencent_cloud.rs
│
├── playbook/               (NEW: AI dialogue engine)
│   ├── mod.rs
│   ├── runner.rs
│   ├── dialogue.rs
│   └── handler/
│       ├── mod.rs
│       ├── provider.rs
│       ├── rag.rs
│       └── types.rs
│
├── offline/                (NEW: embedded ONNX models)
│   ├── mod.rs
│   ├── config.rs
│   ├── downloader.rs
│   ├── sensevoice/
│   └── supertonic/
│
├── useragent/              (NEW: SIP UA for AI agent)
│   ├── mod.rs
│   ├── invitation.rs
│   ├── registration.rs
│   ├── playbook_handler.rs
│   └── webhook.rs
│
├── handler/                (EXTENDED)
│   ├── voice_agent.rs      (NEW: voice agent API routes)
│   └── ... (existing kept)
│
├── addons/                 (KEPT: all existing addons)
├── callrecord/             (KEPT: our richer CDR system)
├── console/                (KEPT: admin web UI)
├── models/                 (KEPT: database entities)
├── proxy/                  (KEPT: SIP proxy core)
├── services/               (KEPT)
├── sipflow/                (KEPT: recording backend)
├── storage/                (KEPT: object storage)
├── voicemail/              (KEPT: voicemail system)
└── bin/
    ├── rustpbx.rs          (KEPT)
    └── sipflow.rs          (KEPT)
```

---

## Risk Register

| Risk | Mitigation |
|------|-----------|
| Track trait rename breaks proxy code | Search-replace carefully, run all proxy tests |
| rand 0.8 vs 0.10 API differences | Port active-call code to rand 0.10 API |
| ONNX Runtime build complexity | Use `load-dynamic` feature, defer to runtime |
| AppState shape mismatch | Create VoiceAgentState wrapper, don't force into our AppState |
| Binary size increase (~50MB with ONNX) | Feature-gate `offline` behind opt-in flag |
| Minijinja version/usage conflicts | Already in deps for console; playbook can reuse |

---

## Success Criteria

1. `cargo check` passes with `--features default` (voice-agent without offline)
2. `cargo check` passes with `--features "default,offline"` (full AI stack)
3. All existing `cargo test` tests pass (no PBX regressions)
4. New playbook example can be loaded and parsed
5. StreamEngine can register and create VAD/ASR/TTS processors
6. Voice agent HTTP routes respond (even if no calls active)
