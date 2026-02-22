# Feature Overlap & Provider Decision Matrix

## Purpose
This document maps every capability required by the new VoIP PBX platform against what is natively provided by **RustPBX**, **Telnyx**, and the legacy **CTM/Twilio** stack. Where two or more providers offer the same functionality, a decision is needed on which provider should own that responsibility.

---

## Reading the Matrix

| Symbol | Meaning |
|--------|---------|
| ✅ | Fully supported, production-ready |
| ⚠️ | Partially supported or requires configuration |
| 🔧 | Requires custom development or third-party integration |
| ❌ | Not available |
| **⚡ OVERLAP** | Two or more providers offer this — decision required |

---

## 1. SIP Trunking & PSTN Connectivity

| Capability | RustPBX | Telnyx | Notes |
|-----------|---------|--------|-------|
| PSTN origination (inbound DID) | ❌ | ✅ | Telnyx owns the carrier relationship — no overlap |
| PSTN termination (outbound) | ❌ | ✅ | Telnyx owns the carrier relationship — no overlap |
| SIP trunk registration | ✅ | ✅ | RustPBX registers to Telnyx as a SIP client |
| Phone number provisioning | ❌ | ✅ | Telnyx Mission Control Portal or API |
| Number porting (from Twilio) | ❌ | ✅ | Telnyx handles port-in requests |
| E911 compliance | ❌ | ✅ | Telnyx provides Kari's Law / Ray Baum compliance |
| STIR/SHAKEN attestation | ❌ | ✅ | Telnyx signs outbound calls on their network |

**Decision**: No overlap here. Telnyx is the carrier; RustPBX is the PBX. Clean separation.

---

## 2. Call Routing & Dialplan

| Capability | RustPBX | Telnyx | Notes |
|-----------|---------|--------|-------|
| **⚡ Inbound call routing** | ✅ Pattern-based dialplan | ✅ Call Control API webhooks | **OVERLAP** |
| **⚡ IVR / auto-attendant** | ⚠️ DTMF + dialplan patterns | ✅ Call Control gather/speak commands | **OVERLAP** |
| **⚡ Call transfer (blind/attended)** | ✅ SIP REFER + B2BUA | ✅ Call Control transfer command | **OVERLAP** |
| **⚡ Call queuing** | ✅ QueuePlan with distribution | ⚠️ Call Control park + custom logic | **OVERLAP** |
| Extension-to-extension (internal) | ✅ Registrar + dialplan | ❌ | RustPBX only |
| Time-based routing | ⚠️ Dialplan patterns | ✅ Webhook logic | Both possible |
| ACL / call filtering | ✅ ACL module with CIDR rules | ✅ Connection-level filtering | Both layers useful |

### 🔴 Decision Required: Call Routing

**Option A — RustPBX owns routing (RECOMMENDED)**
- Use Telnyx in pure SIP trunking mode (no Call Control API)
- All routing logic lives in RustPBX dialplan files
- Telnyx simply delivers inbound SIP to RustPBX and terminates outbound SIP
- Pros: Single point of routing logic, full control, no webhook latency, works offline
- Cons: Must build all IVR/routing in RustPBX config files

**Option B — Telnyx Call Control owns routing**
- Use Telnyx Voice API (programmable voice) instead of raw SIP trunking
- Routing decisions made via webhook callbacks to your application server
- Pros: Rich API, built-in TTS for IVR prompts, no PBX needed for simple flows
- Cons: Adds latency (webhook round-trip), splits routing logic across two systems, requires always-on webhook server

**Option C — Hybrid**
- Telnyx handles first-level IVR (press 1 for sales, 2 for support) via Call Control
- Then bridges to RustPBX for extension routing, queuing, agent distribution
- Cons: Most complex to maintain; debugging spans two systems

> **⚠️ IMPORTANT**: Telnyx SIP Trunking and Telnyx Call Control (Voice API) are *mutually exclusive connection types*. A SIP Connection configured with a webhook URL is treated as programmable voice, NOT SIP trunking. You must choose one mode per connection. You can, however, have separate connections for different numbers.

---

## 3. Call Recording

| Capability | RustPBX | Telnyx | Notes |
|-----------|---------|--------|-------|
| **⚡ Call recording (start/stop)** | ✅ SipFlow (unified SIP+RTP capture) | ✅ Call Control `record_start` API | **OVERLAP** |
| **⚡ Recording storage** | ✅ Local filesystem + SQLite | ✅ Cloud (S3-based, accessible via API) | **OVERLAP** |
| Dual-channel (stereo) recording | ✅ SipFlow captures both legs | ✅ `channels: "dual"` parameter | Both support |
| Recording format | ✅ WAV, OGG/Opus | ✅ WAV, MP3 | Slight format differences |
| Pause/resume recording | 🔧 Would need custom dev | ✅ Call Control API | Telnyx richer here |
| Recording retention/lifecycle | ⚠️ Manual or addon-archive | ✅ Managed cloud storage | Telnyx simpler |
| Recording access API | ✅ AMI API + filesystem | ✅ REST API with signed URLs | Both accessible |

### 🔴 Decision Required: Call Recording

**Option A — RustPBX SipFlow recording (RECOMMENDED for control)**
- SipFlow captures unified SIP signaling + RTP media locally
- Recordings stored on your infrastructure — full data sovereignty
- No per-minute recording fees from Telnyx
- Pros: No cloud dependency, no extra cost per minute, full packet capture for debugging
- Cons: You manage storage, backups, and retention; must build your own playback UI

**Option B — Telnyx cloud recording**
- Telnyx records in their cloud, provides download URLs
- Only available if using Call Control (Voice API) mode — NOT available in raw SIP trunking mode
- Pros: Zero infrastructure, built-in storage management
- Cons: Per-minute recording cost ($0.002/min+), data lives in Telnyx cloud, requires Voice API mode

**Option C — Both (belt and suspenders)**
- RustPBX captures locally via SipFlow for debugging and compliance archive
- Telnyx cloud recording for easy web access and sharing
- Cons: Double storage, double cost; only possible in Voice API mode

> **⚠️ KEY CONSTRAINT**: Telnyx call recording is only available via the Call Control / Voice API. If you choose pure SIP trunking mode (Option A in routing above), Telnyx recording is NOT available — RustPBX must handle it.

---

## 4. Call Transcription (Speech-to-Text)

| Capability | RustPBX | Telnyx | Notes |
|-----------|---------|--------|-------|
| **⚡ Post-call transcription** | ⚠️ addon-transcript (ASR providers) | ✅ `transcription="true"` on Record verb | **OVERLAP** |
| **⚡ Real-time transcription** | ⚠️ ASR pipeline (Tencent, Aliyun, Deepgram) | ✅ `transcription_start` command | **OVERLAP** |
| ASR engine options | Tencent, Aliyun, Deepgram, VoiceAPI | Telnyx native ($0.025/min) or Google ($0.05/min) | Different provider pools |
| Speaker diarization | 🔧 Depends on ASR provider | ✅ Automatic with dual-channel | Telnyx easier |
| Streaming/interim results | ⚠️ Provider-dependent | ✅ Google engine supports interim | Telnyx more mature |
| Webhook delivery of transcript | 🔧 Custom development | ✅ Built-in `transcriptionCallback` | Telnyx simpler |

### 🔴 Decision Required: Call Transcription

**Option A — RustPBX addon-transcript + third-party ASR (RECOMMENDED for flexibility)**
- RustPBX captures audio via SipFlow, feeds to ASR provider (Deepgram recommended)
- Deepgram pricing: ~$0.0043/min (Nova-2) — significantly cheaper than Telnyx
- Full control over ASR provider choice, model tuning, custom vocabulary
- Pros: Lowest cost, best accuracy (Deepgram Nova-2), provider flexibility, data sovereignty
- Cons: More integration work, must build transcript storage/retrieval

**Option B — Telnyx built-in transcription**
- Simple: set `transcription="true"` and receive webhooks with text
- Telnyx engine: $0.025/min; Google engine: $0.05/min
- Only available in Call Control / Voice API mode
- Pros: Fastest to implement, no additional accounts needed
- Cons: 5-10x more expensive than Deepgram, locked to Telnyx/Google engines, requires Voice API mode

**Option C — Hybrid (real-time via Telnyx, post-call via Deepgram)**
- Use Telnyx real-time transcription for live agent coaching during calls
- Use Deepgram for post-call batch transcription (cheaper, more accurate for archives)
- Cons: Two transcription systems to maintain; requires Voice API mode for the real-time piece

> **💡 NOTE**: RustPBX's voice agent functionality (full ASR→LLM→TTS pipeline) has been moved to a separate repo called **Active Call** (https://github.com/restsend). The addon-transcript feature in RustPBX itself focuses on post-call transcription triggered by a command/script after recording completes.

---

## 5. Text-to-Speech (TTS) & Voice Prompts

| Capability | RustPBX | Telnyx | Notes |
|-----------|---------|--------|-------|
| **⚡ IVR voice prompts** | ⚠️ Pre-recorded audio files | ✅ TTS via Amazon Polly (29 languages) + KokoroTTS | **OVERLAP** |
| Dynamic TTS (read text aloud) | 🔧 Via Active Call / external TTS | ✅ `<Say>` verb or `speak` command | Telnyx far easier |
| Pre-recorded audio playback | ✅ Media file playback | ✅ `<Play>` verb or `play_audio` command | Both support |

### Decision: TTS / Voice Prompts

This is less of a conflict and more of a capability gap. RustPBX in pure PBX mode relies on pre-recorded audio files for prompts. If you need dynamic TTS (reading caller names, account numbers, custom messages), you'd either need Telnyx Call Control or an external TTS service integrated with RustPBX. For a self-hosted PBX approach, pre-recorded prompts are typical and sufficient for most IVR scenarios.

---

## 6. Call Scripting / Agent Coaching

| Capability | RustPBX | Telnyx | Notes |
|-----------|---------|--------|-------|
| Real-time media streaming | ⚠️ SipFlow captures, not real-time fork | ✅ Media Forking (WebSocket stream) | Telnyx stronger |
| Live agent whisper/coaching | 🔧 Custom development needed | ⚠️ Media fork + external AI processing | Both need custom work |
| Post-call analysis/scoring | 🔧 Custom (transcript + LLM) | 🔧 Custom (transcript + LLM) | Neither has this built-in |
| Screen pop / CRM integration | 🔧 Via AMI API webhooks | 🔧 Via Call Control webhooks | Both need custom dev |

### Decision: Agent Coaching

Neither platform provides turnkey agent coaching. This will be custom-built regardless. The key architectural question is where the real-time audio stream comes from:
- **Telnyx Media Forking** can duplicate live audio to a WebSocket endpoint for real-time ASR + LLM analysis
- **RustPBX** would require intercepting the RTP stream at the media proxy level

Telnyx Media Forking is the more proven path for real-time coaching. Post-call coaching (analyzing transcripts after the fact) can be done entirely with recordings from either platform + an LLM.

---

## 7. Voicemail

| Capability | RustPBX | Telnyx | Notes |
|-----------|---------|--------|-------|
| Voicemail system | 🔧 No built-in voicemail module | ⚠️ Can build with Record verb + timeout | Neither has turnkey VM |
| Voicemail-to-email | 🔧 Custom development | 🔧 Custom development | Both need work |
| Voicemail transcription | 🔧 Recording + ASR pipeline | ✅ Record verb with `transcription="true"` | Telnyx easier |
| Greeting management | 🔧 Custom (audio file per extension) | ⚠️ TTS or Play verb per call flow | Both need work |
| MWI (Message Waiting Indicator) | 🔧 SIP NOTIFY implementation needed | ❌ Not applicable (no SIP phones) | PBX-level feature |

### Decision: Voicemail

Voicemail is a gap in both platforms — neither provides a complete voicemail system out of the box. This will be custom development. The recommended approach is to build it within RustPBX's dialplan:

1. No-answer timeout → route to voicemail dialplan
2. Play greeting (pre-recorded audio file per extension)
3. Record message using RustPBX's recording system
4. Post-recording: trigger transcription via Deepgram, email notification with audio + transcript
5. Store in database with read/unread status, accessible via web console

---

## 8. WebRTC / SoftPhone Support

| Capability | RustPBX | Telnyx | Notes |
|-----------|---------|--------|-------|
| **⚡ WebRTC gateway** | ✅ Built-in WebSocket SIP + browser client | ✅ Telnyx WebRTC SDK (browser/mobile) | **OVERLAP** |
| SIP softphone support | ✅ Standard SIP registration | ✅ Register to sip.telnyx.com | **OVERLAP** |
| Browser-based phone | ✅ `/console` includes WebRTC client | ✅ Telnyx WebRTC JavaScript SDK | **OVERLAP** |
| Mobile SDK | ❌ | ✅ iOS + Android SDKs | Telnyx only |

### 🔴 Decision Required: SoftPhone Connectivity

**Option A — SoftPhones register to RustPBX (RECOMMENDED)**
- SIP softphones (Ooma, Zoiper, Ooma, MicroSIP, etc.) register directly to RustPBX
- Browser clients use RustPBX's built-in WebRTC gateway
- RustPBX handles all internal routing, then trunks external calls to Telnyx
- Pros: Full PBX features (hold, transfer, park, intercom), internal extension dialing, centralized control
- Cons: RustPBX must be reachable by all softphones (VPN or public IP with TLS)

**Option B — SoftPhones register to Telnyx**
- SIP softphones register to `sip.telnyx.com` using Telnyx credentials
- Telnyx handles all call routing
- Pros: No PBX infrastructure needed for basic calling
- Cons: No internal extensions, no queue-based routing, no PBX features — defeats purpose of RustPBX

> **Recommendation**: Option A. The whole point of deploying RustPBX is to be the call control point. Softphones should register to RustPBX, and RustPBX trunks to Telnyx for PSTN connectivity.

---

## 9. CDR / Call Analytics

| Capability | RustPBX | Telnyx | Notes |
|-----------|---------|--------|-------|
| **⚡ Call Detail Records** | ✅ Async CDR pipeline + database | ✅ CDR API + Mission Control Portal | **OVERLAP** |
| Call quality metrics (QoS) | ⚠️ SipFlow captures RTCP | ✅ Webhook includes MOS, jitter, packet loss | Both available |
| Reporting dashboard | ✅ Web console CDR viewer | ✅ Mission Control Portal analytics | Both available |
| CDR export | ✅ S3, HTTP webhook, database | ✅ CSV export, API access | Both available |
| Real-time call monitoring | ✅ AMI API `/ami/v1` endpoints | ✅ Call Control webhooks | Both available |

### Decision: CDR / Analytics

Keep both. They serve different purposes:
- **RustPBX CDR**: Internal PBX-level detail (extension, queue, ring time, agent who answered, internal transfers)
- **Telnyx CDR**: Carrier-level detail (PSTN cost, DID usage, trunk utilization, network quality)

These are complementary, not redundant. Your reporting system should pull from both sources.

---

## 10. Administration & Management

| Capability | RustPBX | Telnyx | Notes |
|-----------|---------|--------|-------|
| Web admin portal | ✅ `/console` (Alpine.js UI) | ✅ Mission Control Portal | Different scopes |
| REST API | ✅ AMI API `/ami/v1` | ✅ Telnyx v2 API | Different scopes |
| Hot reload (no restart) | ✅ Routes, trunks, ACL via API | N/A | RustPBX advantage |
| Number management | ❌ | ✅ Buy, port, configure DIDs | Telnyx only |
| User/extension management | ✅ Multiple user backends | ❌ | RustPBX only |
| Billing/spend controls | ❌ | ✅ Daily spend limits, destination restrictions | Telnyx only |

### Decision: Administration

No real overlap — these manage different layers. Use both:
- **Telnyx Mission Control**: Carrier-level management (numbers, trunks, spend limits)
- **RustPBX Console**: PBX-level management (extensions, routes, queues, recordings)

---

## Summary: The Five Decisions You Need to Make

| # | Overlap Area | Option A (RustPBX) | Option B (Telnyx) | Recommendation |
|---|-------------|-------------------|------------------|----------------|
| 1 | **Call Routing / IVR** | Dialplan patterns + QueuePlan | Call Control webhooks | **RustPBX** — single routing authority, no webhook latency |
| 2 | **Call Recording** | SipFlow local capture | Cloud recording via API | **RustPBX** — data sovereignty, no per-min cost, works with SIP trunk mode |
| 3 | **Transcription** | addon-transcript + Deepgram | Built-in STT ($0.025-0.05/min) | **RustPBX + Deepgram** — 5-10x cheaper, better accuracy, provider flexibility |
| 4 | **WebRTC / SoftPhone** | Built-in WebSocket gateway | Telnyx WebRTC SDK | **RustPBX** — centralizes all phone features through PBX |
| 5 | **Agent Coaching** | Custom (post-call via transcript) | Media Forking (real-time stream) | **Hybrid** — post-call via RustPBX recordings + Deepgram; real-time TBD |

---

## The Architectural Consequence

These five decisions are interconnected because of a critical Telnyx constraint:

> **Telnyx SIP Trunking and Voice API (Call Control) are mutually exclusive per connection.**

If you choose RustPBX for routing (Decision #1), you are choosing **SIP Trunking mode**, which means:
- ✅ RustPBX handles all call routing, recording, and processing
- ✅ Lowest cost (no Voice API per-minute surcharges)
- ✅ Full PBX feature set (queues, transfers, parking, intercom)
- ❌ Telnyx Call Control features (cloud recording, built-in transcription, media forking, TTS) are NOT available
- ❌ Agent coaching via Telnyx media forking is NOT available in this mode

This means the **recommended architecture** is:

```
┌──────────────┐     SIP Trunk      ┌──────────────┐     SIP      ┌──────────────┐
│              │  (pure transport)   │              │  (register)  │              │
│    Telnyx    │◄──────────────────►│   RustPBX    │◄────────────►│  SoftPhones  │
│   (carrier)  │   UDP/TCP/TLS     │    (PBX)     │  UDP/WS/TLS  │  (agents)    │
│              │                    │              │              │              │
└──────────────┘                    └──────┬───────┘              └──────────────┘
                                          │
                                    ┌─────┴─────┐
                                    │           │
                               ┌────▼───┐ ┌────▼────┐
                               │SipFlow │ │Deepgram │
                               │Record  │ │  ASR    │
                               │(local) │ │(transcr)│
                               └────────┘ └─────────┘
```

Telnyx's role becomes purely **carrier/transport** — delivering calls to and from the PSTN. All intelligence lives in RustPBX and your custom application layer.

---

## What This Means for Each Legacy CTM Feature

| CTM Feature | New Platform Owner | Implementation Path |
|------------|-------------------|-------------------|
| Inbound call tracking | RustPBX | DID → route mapping in dialplan |
| Call routing / IVR | RustPBX | Pattern-based dialplan + DTMF |
| Call recording | RustPBX | SipFlow with local backend |
| Call transcription | RustPBX + Deepgram | Post-call ASR pipeline |
| Agent whisper/coaching | Custom application | Post-call transcript analysis via LLM; real-time TBD |
| Voicemail | Custom (on RustPBX) | No-answer → record → transcribe → email |
| Reporting / analytics | RustPBX CDR + custom | Database-backed CDR with web dashboard |
| Phone number management | Telnyx | Mission Control Portal / API |
| Outbound dialing | RustPBX → Telnyx trunk | Trunk-based outbound routing |

---

## Open Questions for Discussion

1. **Deployment environment**: Where will RustPBX run? Cloud VM (AWS/GCP/Azure), on-premises, or hybrid? This affects Telnyx auth method (credential vs. IP-based) and NAT traversal.

2. **Real-time agent coaching priority**: If real-time coaching is a must-have (not just post-call analysis), we may need a hybrid approach where specific numbers use Telnyx Voice API mode for media forking, while the rest use SIP trunking through RustPBX.

3. **Scale expectations**: How many concurrent calls? How many agents? This affects whether SQLite is sufficient or MySQL is needed.

4. **SoftPhone client preference**: Any existing softphone in use (Ooma, Zoiper, MicroSIP, browser-only)?

5. **Existing Telnyx configuration**: Do you already have SIP Connections, DIDs, and Outbound Voice Profiles configured, or starting fresh?

---

*Document created: 2026-02-19*
*Project: RustPBX VoIP PBX Platform*
*Location: c:\development\rustpbx\docs\FEATURE_OVERLAP_ANALYSIS.md*
