# RustPBX Functional Test Plan

**Created:** 2026-02-20
**RustPBX Version:** 0.3.18
**Goal:** Validate all core PBX functionality end-to-end, from internal softphone
calls through to external PSTN calls via Telnyx SIP trunking.

---

## Prerequisites

| Requirement | Details |
|-------------|---------|
| Docker | RustPBX container running (`ghcr.io/restsend/rustpbx:latest`) |
| Config | `config.toml` mounted at `/app/config.toml` |
| Admin | Console super-user created (`admin`) |
| Softphone | MicroSIP (Windows) — free, lightweight SIP client |
| Telnyx Account | Mission Control access with a funded balance |
| Phone | Personal mobile for inbound/outbound PSTN call testing |

---

## Phase 1: Infrastructure Verification

Confirms the Docker container is healthy and all services are reachable.

### 1.1 Container Health

| # | Test | Command / Action | Expected Result | Status |
|---|------|-----------------|-----------------|--------|
| 1.1.1 | Container running | `docker ps --filter name=rustpbx` | Status: `Up`, ports 5060+8080 mapped | |
| 1.1.2 | No crash loops | `docker logs rustpbx 2>&1 \| grep -i error` | No ERROR lines (startup migration logs only) | |
| 1.1.3 | Config loaded | `docker logs rustpbx 2>&1 \| head -3` | Shows `Loading config from: /app/config.toml` | |
| 1.1.4 | SIP proxy started | `docker logs rustpbx 2>&1 \| grep "start proxy"` | `start proxy, udp port: UDP ....:5060` | |
| 1.1.5 | HTTP server started | `docker logs rustpbx 2>&1 \| grep "starting rustpbx"` | `starting rustpbx on 0.0.0.0:8080` | |
| 1.1.6 | Modules loaded | `docker logs rustpbx 2>&1 \| grep "modules loaded"` | Lists: acl, auth, presence, registrar, call | |

### 1.2 HTTP & API Endpoints

| # | Test | Command | Expected Result | Status |
|---|------|---------|-----------------|--------|
| 1.2.1 | Landing page | `curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/` | `200` | |
| 1.2.2 | Console login page | `curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/console/login` | `200` | |
| 1.2.3 | Console login POST | `curl -s -X POST -d "identifier=admin&password=admin123" http://localhost:8080/console/login -w "%{http_code}"` | `303` (redirect with session cookie) | |
| 1.2.4 | Console dashboard | `curl -s -b <cookie> http://localhost:8080/console/` | `200`, title contains "Dashboard" | |
| 1.2.5 | Console extensions | `curl -s -b <cookie> http://localhost:8080/console/extensions` | `200` | |
| 1.2.6 | Console routing | `curl -s -b <cookie> http://localhost:8080/console/routing` | `200` | |
| 1.2.7 | Console settings | `curl -s -b <cookie> http://localhost:8080/console/settings` | `200` | |
| 1.2.8 | Console call records | `curl -s -b <cookie> http://localhost:8080/console/call-records` | `200` | |
| 1.2.9 | Console diagnostics | `curl -s -b <cookie> http://localhost:8080/console/diagnostics` | `200` | |
| 1.2.10 | AMI health | `curl -s http://localhost:8080/ami/v1/health` | JSON with `"status":"running"` | |
| 1.2.11 | AMI dialogs | `curl -s http://localhost:8080/ami/v1/dialogs` | `[]` (empty array, no active calls) | |
| 1.2.12 | AMI transactions | `curl -s http://localhost:8080/ami/v1/transactions` | `[]` | |

### 1.3 WebSocket SIP Endpoint

| # | Test | Action | Expected Result | Status |
|---|------|--------|-----------------|--------|
| 1.3.1 | WS upgrade accepted | Connect to `ws://localhost:8080/ws` | `101 Switching Protocols`, log shows "created WebSocket channel connection" | |
| 1.3.2 | WS disconnect clean | Disconnect WebSocket | Log shows "WebSocket connection handler exiting" | |

---

## Phase 2: SIP Registration — Internal Softphone

Tests that SIP clients can register with the PBX using the memory-backend test users.

### 2.0 MicroSIP Installation

1. Download MicroSIP from https://www.microsip.org/downloads
2. Install on the Windows host machine
3. No special plugins required — default install is sufficient

### 2.1 Register Extension 1001

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Open MicroSIP, go to **Account** settings | Account dialog opens |
| 2 | Configure account: | |
| | **Account Name:** `Test 1001` | |
| | **SIP Server:** `127.0.0.1` | |
| | **SIP Proxy:** `127.0.0.1:5060` | |
| | **Username:** `1001` | |
| | **Domain:** `127.0.0.1` | |
| | **Password:** `test1001` | |
| | **Transport:** `UDP` | |
| 3 | Click **Save** | MicroSIP attempts REGISTER |
| 4 | Check MicroSIP status bar | Shows "Online" or green indicator |

**Verification commands:**

| # | Test | Command | Expected Result | Status |
|---|------|---------|-----------------|--------|
| 2.1.1 | REGISTER in logs | `docker logs rustpbx 2>&1 \| grep -i register` | Shows REGISTER transaction for 1001 | |
| 2.1.2 | AMI shows registration | `curl -s http://localhost:8080/ami/v1/health` | `sipserver.dialogs` or registration count > 0 | |
| 2.1.3 | Console shows registered | Browse to `/console/diagnostics` | Extension 1001 appears as registered | |

### 2.2 Register Extension 1002

Repeat 2.1 using a **second instance** of MicroSIP (or a different SIP client) with:
- **Username:** `1002`
- **Password:** `test1002`

| # | Test | Expected Result | Status |
|---|------|-----------------|--------|
| 2.2.1 | 1002 registered | MicroSIP shows "Online" | |
| 2.2.2 | Both extensions visible | Console diagnostics shows both 1001 and 1002 registered | |

---

## Phase 3: Internal Calls — Extension to Extension

Tests that two registered softphones can call each other through the PBX.

### 3.0 Route Configuration for Internal Calls

Before testing, add an internal route. From the **Console > Routing** page, or
add to `config.toml`:

```toml
[[proxy.routes]]
name = "internal"
priority = 100
direction = "any"

[proxy.routes.match]
"to.user" = "^(100[0-9])$"

[proxy.routes.action]
type = "local"

[proxy.routes.rewrite]
"to.host" = "127.0.0.1"
```

After editing, reload routes:
```bash
curl -s -X POST http://localhost:8080/ami/v1/reload/routes
```

### 3.1 Call from 1001 → 1002

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | On 1001 softphone, dial `1002` | Softphone sends INVITE |
| 2 | Observe 1002 softphone | 1002 rings, shows incoming call from 1001 |
| 3 | Answer on 1002 | Call connects, both parties hear audio |
| 4 | Speak on both sides | Confirm two-way audio works |
| 5 | Hang up from either side | Call ends cleanly |

**Verification:**

| # | Test | Command / Action | Expected Result | Status |
|---|------|-----------------|-----------------|--------|
| 3.1.1 | Call in AMI during ring | `curl -s http://localhost:8080/ami/v1/dialogs` | Shows active dialog with call-id | |
| 3.1.2 | Call in AMI after hangup | `curl -s http://localhost:8080/ami/v1/dialogs` | `[]` (empty — call cleared) | |
| 3.1.3 | CDR created | Browse `/console/call-records` | New record: 1001 → 1002, status "completed" | |
| 3.1.4 | Recording file | `docker exec rustpbx sh -c "ls -la /app/config/cdr/"` | WAV or recording file present (if recording enabled) | |

### 3.2 Call from 1002 → 1001

Repeat 3.1 in reverse direction.

| # | Test | Expected Result | Status |
|---|------|-----------------|--------|
| 3.2.1 | 1001 rings when 1002 calls | Bidirectional calling works | |
| 3.2.2 | Two-way audio | Audio flows in both directions | |
| 3.2.3 | CDR created | Second call record appears | |

### 3.3 Edge Cases

| # | Test | Action | Expected Result | Status |
|---|------|--------|-----------------|--------|
| 3.3.1 | Call unregistered extension | Dial `1003` from 1001 | Call fails, appropriate SIP error (404 or 480) | |
| 3.3.2 | Call while busy | 1001 calls 1002 while 1002 is on another call | Busy signal or 486 Busy Here | |
| 3.3.3 | Caller hangup | 1001 hangs up during ringing | 1002 stops ringing, no CDR anomaly | |
| 3.3.4 | Callee reject | 1002 rejects incoming call | 1001 hears busy, CDR shows "rejected" or "no-answer" | |

---

## Phase 4: Telnyx SIP Trunk Setup

Configures the Telnyx carrier for PSTN connectivity.

### 4.1 Telnyx Mission Control — SIP Connection

Log in to https://portal.telnyx.com and perform these steps:

| Step | Action | Details |
|------|--------|---------|
| 1 | **Navigate** to SIP Connections | Networking > SIP Connections |
| 2 | **Create** new SIP Connection | Click "Add SIP Connection" |
| 3 | **Name** the connection | `RustPBX-Dev` (or similar) |
| 4 | **Connection Type** | Credential Authentication |
| 5 | **Generate credentials** | Note the **SIP username** and **SIP password** |
| 6 | **Webhook URL** | **Leave BLANK** — critical for pure SIP trunk mode |
| 7 | **Outbound** section | Ensure outbound calls are enabled |
| 8 | **Codecs** | PCMU, PCMA, G.722 (in priority order) |
| 9 | **Save** the connection | Note the connection ID |

> **Warning:** Setting a webhook URL converts the connection to programmable voice
> mode and adds latency. Leave it blank for direct SIP trunking.

### 4.2 Telnyx Mission Control — Phone Number

| Step | Action | Details |
|------|--------|---------|
| 1 | **Navigate** to Numbers | Numbers > My Numbers, or Numbers > Search & Buy |
| 2 | **Purchase** a DID | Buy a local number (e.g., US +1 area code) |
| 3 | **Assign** to SIP Connection | Edit the number, set Connection to `RustPBX-Dev` |
| 4 | **Note** the number | Record the full E.164 number (e.g., `+12025551234`) |

### 4.3 Telnyx Mission Control — Outbound Voice Profile

| Step | Action | Details |
|------|--------|---------|
| 1 | **Navigate** to Outbound Voice Profiles | Voice > Outbound Voice Profiles |
| 2 | **Create** a profile | Name: `RustPBX-Outbound` |
| 3 | **Assign** the SIP Connection | Link to `RustPBX-Dev` |
| 4 | **Assign** phone number(s) | Add the purchased DID as the outbound caller ID |
| 5 | **Save** | Profile becomes active |

### 4.4 RustPBX Trunk Configuration

Create a trunk config file:

**File: `C:\Development\RustPBX\telnyx-trunk.toml`**

```toml
[proxy.trunks.telnyx]
dest = "sip:sip.telnyx.com:5060"
transport = "udp"
username = "<TELNYX_SIP_USERNAME>"
password = "<TELNYX_SIP_PASSWORD>"
direction = "bidirectional"
codec = ["PCMU", "PCMA"]
max_calls = 50
max_cps = 5
```

Mount this file into the container's `config/trunks/` directory, or add the
trunk directly in `config.toml` under `[proxy.trunks.telnyx]`.

**Option A — Add to config.toml directly:**

```toml
[proxy.trunks.telnyx]
dest = "sip:sip.telnyx.com:5060"
transport = "udp"
username = "<TELNYX_SIP_USERNAME>"
password = "<TELNYX_SIP_PASSWORD>"
direction = "bidirectional"
codec = ["PCMU", "PCMA"]
```

**Option B — Separate file in trunks directory:**

Mount the file and hot-reload:
```bash
curl -s -X POST http://localhost:8080/ami/v1/reload/trunks
```

### 4.5 RustPBX Route Configuration for Telnyx

Add routes for inbound (PSTN → extension) and outbound (extension → PSTN):

```toml
# Inbound: calls from Telnyx trunk route to extension 1001
[[proxy.routes]]
name = "telnyx-inbound"
priority = 50
direction = "inbound"
dest = "local"

[proxy.routes.match]
"to.user" = "<TELNYX_DID_NUMBER>"    # e.g., "12025551234" (without +)

[proxy.routes.rewrite]
"to.user" = "1001"
"to.host" = "127.0.0.1"

# Outbound: 10+ digit calls from extensions route to Telnyx
[[proxy.routes]]
name = "telnyx-outbound"
priority = 50
direction = "outbound"
dest = "telnyx"

[proxy.routes.match]
"to.user" = "^\\+?1?[2-9]\\d{9}$"   # North American 10-digit

[proxy.routes.rewrite]
"from.user" = "<TELNYX_DID_NUMBER>"   # Set outbound caller ID
"from.host" = "sip.telnyx.com"
```

After adding routes, reload:
```bash
curl -s -X POST http://localhost:8080/ami/v1/reload/routes
curl -s -X POST http://localhost:8080/ami/v1/reload/trunks
```

### 4.6 Verify Trunk Configuration

| # | Test | Command / Action | Expected Result | Status |
|---|------|-----------------|-----------------|--------|
| 4.6.1 | Trunk loaded | `docker logs rustpbx 2>&1 \| grep "trunks reloaded"` | `trunks reloaded total=1` | |
| 4.6.2 | Routes loaded | `docker logs rustpbx 2>&1 \| grep "routes reloaded"` | `routes reloaded total=2+` (internal + telnyx routes) | |
| 4.6.3 | Console shows trunk | Browse `/console/settings` or routing page | Telnyx trunk visible in configuration | |
| 4.6.4 | AMI health still OK | `curl -s http://localhost:8080/ami/v1/health` | `"status":"running"`, no errors | |

---

## Phase 5: Outbound PSTN Calls (PBX → Phone)

Tests that a registered softphone can call an external phone number via Telnyx.

### 5.1 Outbound Call from Extension

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | On 1001 softphone, dial your mobile number (e.g., `+1XXXXXXXXXX`) | Softphone sends INVITE |
| 2 | Observe RustPBX logs | Shows route match to `telnyx-outbound`, INVITE forwarded to `sip.telnyx.com` |
| 3 | Mobile phone rings | Caller ID shows the Telnyx DID number |
| 4 | Answer mobile phone | Call connects, two-way audio |
| 5 | Speak on both sides | Confirm audio flows bidirectionally |
| 6 | Hang up from either side | Call ends cleanly |

**Verification:**

| # | Test | Command / Action | Expected Result | Status |
|---|------|-----------------|-----------------|--------|
| 5.1.1 | INVITE to Telnyx in logs | `docker logs rustpbx 2>&1 \| grep -i telnyx` | INVITE sent to sip.telnyx.com | |
| 5.1.2 | Call active in AMI | `curl -s http://localhost:8080/ami/v1/dialogs` | Active dialog during call | |
| 5.1.3 | CDR after hangup | Browse `/console/call-records` | Record: direction "outbound", from 1001, to mobile number | |
| 5.1.4 | Telnyx portal CDR | Check Telnyx Mission Control > Call Events | Call logged with matching timestamps | |

### 5.2 Outbound Call Failures

| # | Test | Action | Expected Result | Status |
|---|------|--------|-----------------|--------|
| 5.2.1 | Invalid number | Dial `0000000000` from 1001 | Call fails with SIP error (e.g., 404, 503) | |
| 5.2.2 | No route match | Dial `999` (short number, no route) | Call fails, no INVITE sent to Telnyx | |
| 5.2.3 | CDR for failed call | Check call records | Failed call recorded with error status | |

---

## Phase 6: Inbound PSTN Calls (Phone → PBX)

Tests that an external phone calling the Telnyx DID reaches a registered extension.

### 6.0 Network Prerequisites

For inbound calls to reach the Docker container, the PBX must be reachable from
Telnyx. Options:

| Approach | When to Use |
|----------|-------------|
| **Port forwarding** | Home/office router forwards UDP 5060 + RTP range to host |
| **Cloud VM** | Run Docker on a VPS with public IP (simplest) |
| **ngrok/tunneling** | Quick dev testing (SIP over TCP/TLS only) |
| **Telnyx FQDN** | Configure `external_ip` in config.toml to match your public IP |

**Required config.toml addition for NAT:**
```toml
# Add to top level if behind NAT
external_ip = "<YOUR_PUBLIC_IP>"
rtp_start_port = 20000
rtp_end_port = 30000
```

Also ensure Docker exposes the RTP port range:
```bash
docker run ... -p 20000-30000:20000-30000/udp ...
```

### 6.1 Inbound Call to DID

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | From your mobile, dial the Telnyx DID number | Mobile initiates PSTN call |
| 2 | Observe RustPBX logs | Shows incoming INVITE from Telnyx, route match to `telnyx-inbound` |
| 3 | 1001 softphone rings | Incoming call displayed with caller ID of mobile number |
| 4 | Answer on 1001 | Call connects |
| 5 | Speak on both sides | Confirm two-way audio |
| 6 | Hang up | Call ends cleanly |

**Verification:**

| # | Test | Command / Action | Expected Result | Status |
|---|------|-----------------|-----------------|--------|
| 6.1.1 | Inbound INVITE in logs | `docker logs rustpbx 2>&1 \| tail -50` | INVITE from Telnyx IP, matched inbound route | |
| 6.1.2 | CDR after hangup | Browse `/console/call-records` | Record: direction "inbound", from mobile, to DID/1001 | |
| 6.1.3 | Telnyx CDR matches | Check Telnyx Mission Control | Inbound call logged | |

### 6.2 Inbound When Extension Unavailable

| # | Test | Action | Expected Result | Status |
|---|------|--------|-----------------|--------|
| 6.2.1 | Unregistered extension | Unregister 1001, call DID from mobile | Call fails (480 Temporarily Unavailable or similar) | |
| 6.2.2 | Re-register and retry | Register 1001 again, call DID | Call succeeds normally | |

---

## Phase 7: Call Recording Verification

Validates that call recordings are captured and stored.

| # | Test | Action | Expected Result | Status |
|---|------|--------|-----------------|--------|
| 7.1 | Recording config active | `docker logs rustpbx 2>&1 \| grep -i record` | Recording/CallRecordManager serving | |
| 7.2 | Make a test call | Call 1001 → 1002, talk for 10 seconds, hang up | | |
| 7.3 | Recording file exists | `docker exec rustpbx sh -c "find /app/config/cdr/ -type f"` | WAV/recording file present | |
| 7.4 | Recording has audio | Copy file out: `docker cp rustpbx:/app/config/cdr/<file> .` and play | Audio of both parties audible | |
| 7.5 | Recording in CDR | Browse `/console/call-records`, click the call | Recording link/player available | |

---

## Phase 8: CDR (Call Detail Records) Verification

| # | Test | Action | Expected Result | Status |
|---|------|--------|-----------------|--------|
| 8.1 | CDR list populated | Browse `/console/call-records` | All test calls visible | |
| 8.2 | CDR fields correct | Click a call record | Shows: call_id, direction, from, to, duration, status, timestamps | |
| 8.3 | CDR for internal call | Check 1001→1002 record | Direction: "internal" or "outbound", correct extension numbers | |
| 8.4 | CDR for outbound PSTN | Check outbound call record | Direction: "outbound", trunk: telnyx, to: mobile number | |
| 8.5 | CDR for inbound PSTN | Check inbound call record | Direction: "inbound", from: mobile, to: DID/extension | |
| 8.6 | CDR timing accuracy | Compare CDR duration with actual call | Duration within 1-2 seconds of actual | |

---

## Phase 9: Console UI Functional Checks

Validates the web console works correctly in a browser (not just curl).

| # | Test | Action | Expected Result | Status |
|---|------|--------|-----------------|--------|
| 9.1 | Login flow | Browse `http://localhost:8080/console/login`, enter admin/admin123 | Redirected to dashboard | |
| 9.2 | Dashboard loads | Check dashboard page | Shows system stats, call summary, uptime | |
| 9.3 | Extensions page | Navigate to Extensions | Lists configured extensions (1001, 1002) | |
| 9.4 | Create extension via UI | Add extension 1003 with password via console | Extension saved, appears in list | |
| 9.5 | Routing page | Navigate to Routing | Shows configured routes | |
| 9.6 | Create route via UI | Add a test route through the console | Route saved, appears in list | |
| 9.7 | Call records page | Navigate to Call Records | Shows test call CDRs with filtering | |
| 9.8 | Call record detail | Click a specific call record | Full detail view with all fields | |
| 9.9 | Diagnostics page | Navigate to Diagnostics | Shows SIP registration status, system health | |
| 9.10 | Settings page | Navigate to Settings | Shows current configuration, AMI endpoint | |

---

## Phase 10: WebRTC Browser Softphone

Tests the built-in browser-based SIP client (SIP-over-WebSocket).

| # | Test | Action | Expected Result | Status |
|---|------|--------|-----------------|--------|
| 10.1 | Load JsSIP phone page | Browse `http://localhost:8080/static/phone_jssip.html` (or access via console) | Phone UI renders | |
| 10.2 | Configure & register | Enter extension 1002 credentials, server `ws://localhost:8080/ws` | Shows registered status | |
| 10.3 | Receive call from 1001 | 1001 (MicroSIP) calls 1002 (browser) | Browser phone rings | |
| 10.4 | Answer in browser | Click answer | Two-way audio between MicroSIP and browser | |
| 10.5 | Call from browser | Browser phone dials 1001 | MicroSIP rings, answer, two-way audio | |

> **Note:** Browser WebRTC may require STUN/TURN configuration in `config.toml`
> if audio doesn't flow. Add ICE servers if needed:
> ```toml
> [[ice_servers]]
> urls = ["stun:stun.l.google.com:19302"]
> ```

---

## Test Execution Checklist

### Execution Order

The tests should be run in phase order since later phases depend on earlier ones:

```
Phase 1 (Infrastructure)
  └─► Phase 2 (SIP Registration)
        └─► Phase 3 (Internal Calls)
              └─► Phase 4 (Telnyx Setup)
                    ├─► Phase 5 (Outbound PSTN)
                    └─► Phase 6 (Inbound PSTN)
                          └─► Phase 7 (Recording)
                                └─► Phase 8 (CDR)
Phase 9 (Console UI) — can run in parallel after Phase 3
Phase 10 (WebRTC) — can run in parallel after Phase 2
```

### Summary Scorecard

| Phase | Description | Tests | Passed | Failed | Blocked |
|-------|-------------|-------|--------|--------|---------|
| 1 | Infrastructure | 14 | | | |
| 2 | SIP Registration | 5 | | | |
| 3 | Internal Calls | 10 | | | |
| 4 | Telnyx Setup | 4 | | | |
| 5 | Outbound PSTN | 6 | | | |
| 6 | Inbound PSTN | 5 | | | |
| 7 | Recording | 5 | | | |
| 8 | CDR | 6 | | | |
| 9 | Console UI | 10 | | | |
| 10 | WebRTC | 5 | | | |
| **Total** | | **70** | | | |

---

## Appendix A: Quick Reference — Config Files

### Current config.toml (running)

```toml
http_addr = "0.0.0.0:8080"
log_level = "info"
media_cache_path = "./config/mediacache"
database_url = "sqlite://rustpbx.sqlite3"

[console]
base_path = "/console"
allow_registration = false
secure_cookie = false

[ami]
allows = ["*"]

[proxy]
modules = ["acl", "auth", "presence", "registrar", "call"]
addr = "0.0.0.0"
udp_port = 5060
registrar_expires = 60
ws_handler = "/ws"
media_proxy = "auto"
generated_dir = "./config"
routes_files = ["config/routes/*.toml"]
trunks_files = ["config/trunks/*.toml"]
ensure_user = true

acl_rules = [
    "allow all",
    "deny all"
]

[[proxy.user_backends]]
type = "memory"
users = [
    { username = "1001", password = "test1001" },
    { username = "1002", password = "test1002" },
]

[[proxy.user_backends]]
type = "extension"

[recording]
enabled = true
auto_start = true

[callrecord]
type = "local"
root = "./config/cdr"
```

### Docker Run Command

```bash
MSYS_NO_PATHCONV=1 docker run -d \
  --name rustpbx \
  -p 5060:5060/udp \
  -p 5060:5060/tcp \
  -p 8080:8080 \
  -v "C:\Development\RustPBX\config.toml:/app/config.toml" \
  ghcr.io/restsend/rustpbx:latest \
  --conf /app/config.toml
```

Create admin user (run once after each fresh container start):
```bash
MSYS_NO_PATHCONV=1 docker exec rustpbx \
  /app/rustpbx --conf /app/config.toml \
  --super-username admin --super-password admin123 \
  --super-email admin@rustpbx.local
```

### TrunkConfig Fields Reference

| Field | Type | Description |
|-------|------|-------------|
| `dest` | string | SIP URI (e.g., `sip:sip.telnyx.com:5060`) |
| `backup_dest` | string? | Failover SIP URI |
| `username` | string? | SIP auth username |
| `password` | string? | SIP auth password |
| `transport` | string? | `udp`, `tcp`, or `tls` |
| `direction` | string? | `inbound`, `outbound`, or `bidirectional` |
| `codec` | string[] | Codec priority list |
| `max_calls` | int? | Maximum concurrent calls |
| `max_cps` | int? | Maximum calls per second |
| `inbound_hosts` | string[] | IP whitelist for inbound |

### AMI Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/ami/v1/health` | System health + version |
| GET | `/ami/v1/dialogs` | Active calls |
| GET | `/ami/v1/transactions` | SIP transactions |
| GET | `/ami/v1/hangup/{id}` | Terminate a call |
| POST | `/ami/v1/reload/trunks` | Hot-reload trunk config |
| POST | `/ami/v1/reload/routes` | Hot-reload route config |
| POST | `/ami/v1/reload/acl` | Hot-reload ACL rules |
| POST | `/ami/v1/reload/app` | Full app reload |

---

*Document: `C:\Development\RustPBX\docs\FUNCTIONAL_TESTS.md`*
*Last updated: 2026-02-20*
