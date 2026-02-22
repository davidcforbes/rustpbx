# RustPBX VoIP Platform — Project Overview

## Project Name
**RustPBX VoIP PBX Platform** — A modern, AI-capable PBX built on the RustPBX open-source engine, connected to Telnyx SIP trunking.

## Objective
Replace the legacy CallTrackingMetrics (CTM) platform (which uses Twilio SIP trunking) with a self-hosted, high-performance VoIP PBX solution that provides:

- Inbound and outbound calling via Telnyx SIP trunks
- SoftPhone connectivity (WebRTC + SIP clients)
- Intelligent call routing with queue-based distribution
- Call recording with configurable policies
- Call transcription (speech-to-text)
- Call scripting / agent coaching (real-time or post-call)
- Voicemail with message management

## Technology Stack

| Component | Technology | Notes |
|-----------|-----------|-------|
| PBX Engine | [RustPBX](https://github.com/restsend/rustpbx) (v0.3.15+) | Rust-based, high-performance SIP B2BUA |
| SIP Trunking | Telnyx | Replaces Twilio; credential or IP-based auth |
| Database | SQLite (dev) / MySQL (prod) | Via sea-orm ORM |
| Web Admin | Built-in Console (`/console`) | Alpine.js + MiniJinja templates |
| Management API | AMI REST API (`/ami/v1`) | Programmatic control, hot reload |
| Runtime | Tokio async runtime | Concurrent SIP + HTTP + WebSocket servers |
| Containerization | Docker | `ghcr.io/restsend/rustpbx:latest` |

## Source Repository
- **Upstream**: https://github.com/restsend/rustpbx
- **Documentation Wiki**: https://deepwiki.com/restsend/rustpbx
- **Crate**: https://crates.io/crates/rustpbx
- **Website**: https://www.rustpbx.com/

## RustPBX Core Capabilities (Built-In)

### SIP Proxy & Registration
- Full SIP stack via `rsipstack` library
- Modular pipeline: ACL → Auth → Registrar → Call modules
- Multi-transport: UDP, TCP, TLS, WebSocket
- NAT traversal with automatic media proxy detection

### Call Processing (B2BUA)
- Dialplan resolution with pattern-based route matching
- Call direction classification: Inbound / Outbound / Internal
- Sequential and parallel dialing strategies
- Queue-based call distribution (`QueuePlan`)
- Trunk-based outbound routing

### Media Processing
- RTP/RTCP relay via `MediaBridge`
- Codec support: PCMU, PCMA, Opus, G.722
- Dynamic transcoding between call legs
- WebRTC ↔ SIP interoperability
- DTMF detection and generation

### Recording Systems
- **Legacy Recorder**: Per-call WAV file recording
- **SipFlow** (recommended): Unified SIP + RTP capture with superior I/O
  - Open-write-close pattern (no FD exhaustion)
  - Local backend (filesystem + SQLite) or Remote (distributed UDP)
  - Date-based organization (hourly/daily)

### Call Detail Records (CDR)
- Async pipeline via `mpsc` channels
- Pluggable storage backends: Local, S3, HTTP
- Database persistence via `DatabaseHook`
- Custom formatters and webhook hooks

### Administration
- **Web Console** (`/console`): Extensions, routes, trunks, CDR viewer
- **AMI API** (`/ami/v1`): RESTful programmatic control
  - Hot reload: routes, trunks, ACL rules without restart
- **WebSocket**: Browser-based SIP client support

### Addon System (Cargo Features)
| Addon | Feature Flag | Purpose |
|-------|-------------|---------|
| ACME | `addon-acme` | Automatic SSL certificate management |
| Wholesale | `addon-wholesale` | Billing and wholesale routing |
| Archive | `addon-archive` | Automated call record archival |
| **Transcript** | `addon-transcript` | **Speech-to-text transcription** |

### User Backend System
- Memory (static config), Database, Extension (console-managed)
- HTTP (external API), Plain text file
- Chained backends with priority ordering

## What Needs to Be Built / Configured

### Phase 1: Core PBX + Telnyx Integration
- [ ] Deploy RustPBX (Docker or bare metal)
- [ ] Configure Telnyx SIP trunk (credential-based auth)
- [ ] Configure inbound routing (DID → extension mapping)
- [ ] Configure outbound routing (extension → Telnyx trunk)
- [ ] Set up SoftPhone connectivity (WebRTC + SIP clients)
- [ ] Configure user extensions and authentication

### Phase 2: Call Recording & CDR
- [ ] Enable SipFlow recording with local backend
- [ ] Configure recording policies (global, per-trunk, per-route)
- [ ] Set up CDR storage and database persistence
- [ ] Implement CDR export/reporting

### Phase 3: Transcription & AI Features
- [ ] Enable `addon-transcript` feature
- [ ] Integrate ASR provider (Deepgram, or OpenAI Whisper)
- [ ] Build post-call transcription pipeline
- [ ] Implement call scripting / agent coaching workflow

### Phase 4: Voicemail & Advanced Routing
- [ ] Implement voicemail system (may require custom development)
- [ ] Build IVR / auto-attendant dialplans
- [ ] Implement intelligent call routing rules
- [ ] Configure queue-based distribution with agent groups

### Phase 5: Migration from CTM/Twilio
- [ ] Port phone numbers from Twilio to Telnyx
- [ ] Migrate call routing rules from CTM
- [ ] Parallel testing period
- [ ] Cutover and decommission legacy platform

## Project Location
- **Documentation**: `c:\development\rustpbx\docs\`
- **Architecture**: `c:\development\rustpbx\docs\architecture\`
