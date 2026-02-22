# RustPBX Advanced Test Specification -- L4 through L8

**Created:** 2026-02-21
**RustPBX Version:** 0.3.18
**Stage:** 7 of 9 (see [TESTING_PLAN_OF_ACTION.md](../TESTING_PLAN_OF_ACTION.md))
**Predecessor:** L0-L3 test levels (Smoke, Infrastructure, API Contract, SIP Functional)

---

## Scope

This document specifies test cases for the five advanced test levels of the RustPBX
VoIP platform:

| Level | Name | Focus | Test Count |
|-------|------|-------|------------|
| L4 | Integration (Telnyx PSTN) | End-to-end calls through Telnyx SIP trunk | 7 |
| L5 | Media Quality | Recording integrity, codec negotiation, RTP, transcription | 5 |
| L6 | Load / Stress | Concurrent registrations, calls, API throughput, long calls | 5 |
| L7 | Failover / Resilience | Container restart, DB reconnect, trunk failover, graceful shutdown | 5 |
| L8 | Security | Auth bypass, fuzzing, injection, rate limiting | 7 |
| | **Total** | | **29** |

---

## Environment

| Parameter | Value |
|-----------|-------|
| Host | Linode VPS `74.207.251.126` |
| Container | `ghcr.io/restsend/rustpbx:latest` (Docker) |
| SIP Port | 5060 (UDP/TCP) |
| HTTP Port | 8080 |
| SIP Trunk | Telnyx (`sip.telnyx.com`, credential auth, UDP) |
| DID | +1 (707) 283-3106 (`17072833106`) |
| Database | SQLite (`sqlite://rustpbx.sqlite3`) |
| Recording | sipflow (local) + WAV recorder (`auto_start = true`) |
| CDR | Local filesystem (`./config/cdr`) |
| Transcript | `addon-transcript` (Groq Whisper API via wrapper) |
| Test Users | `1001`/`test1001`, `1002`/`test1002` |
| Admin | `admin`/`admin123` (console super-user) |

### Required Tools

| Tool | Purpose | Install |
|------|---------|---------|
| SIPp | SIP load generator / scenario runner | `apt install sipp` or Docker image |
| PJSUA | CLI SIP user agent (for scripted calls) | `apt install pjsua` or build from PJSIP |
| pytest | Python test runner | `pip install pytest requests` |
| curl | HTTP API testing | Pre-installed |
| tcpdump | RTP packet capture | `apt install tcpdump` |
| socat | UDP probing | `apt install socat` |
| ffprobe | WAV file analysis | `apt install ffmpeg` |
| nmap | Port scanning (security tests) | `apt install nmap` |
| sipvicious | SIP security scanning | `pip install sipvicious` |

---

## Summary Table -- All Test Cases

| ID | Title | Level | Priority | Automated | Tool |
|----|-------|-------|----------|-----------|------|
| TC-L4-001 | Outbound PSTN call via Telnyx | L4 | P0 | Partial | PJSUA + mobile |
| TC-L4-002 | Inbound PSTN call from Telnyx | L4 | P0 | Partial | mobile + PJSUA |
| TC-L4-003 | Outbound call to invalid number | L4 | P1 | Yes | PJSUA / pytest |
| TC-L4-004 | Telnyx trunk registration/health | L4 | P0 | Yes | pytest / curl |
| TC-L4-005 | Caller ID presentation | L4 | P1 | Partial | PJSUA + mobile |
| TC-L4-006 | PSTN call recording | L4 | P1 | Partial | PJSUA + pytest |
| TC-L4-007 | PSTN call CDR | L4 | P1 | Yes | pytest / curl |
| TC-L5-001 | Recording file format validation | L5 | P1 | Yes | pytest + ffprobe |
| TC-L5-002 | Recording audio content | L5 | P2 | Manual | ffplay / Audacity |
| TC-L5-003 | Codec negotiation match | L5 | P1 | Yes | SIPp / PJSUA |
| TC-L5-004 | RTP stream validation | L5 | P1 | Yes | tcpdump + pytest |
| TC-L5-005 | Transcript generation | L5 | P1 | Yes | pytest / curl |
| TC-L6-001 | Concurrent SIP registrations | L6 | P1 | Yes | SIPp |
| TC-L6-002 | Concurrent calls | L6 | P1 | Yes | SIPp |
| TC-L6-003 | API throughput | L6 | P2 | Yes | pytest / curl |
| TC-L6-004 | Registration storm recovery | L6 | P2 | Yes | SIPp |
| TC-L6-005 | Long duration call | L6 | P2 | Partial | PJSUA + pytest |
| TC-L7-001 | Container restart recovery | L7 | P0 | Yes | pytest / docker |
| TC-L7-002 | Database reconnection | L7 | P1 | Yes | pytest / docker |
| TC-L7-003 | Trunk failover (backup_dest) | L7 | P1 | Yes | SIPp / pytest |
| TC-L7-004 | Call survival during config reload | L7 | P1 | Partial | PJSUA + curl |
| TC-L7-005 | Graceful shutdown (SIGTERM) | L7 | P0 | Yes | pytest / docker |
| TC-L8-001 | SIP auth bypass attempt | L8 | P0 | Yes | SIPp / sipvicious |
| TC-L8-002 | Console auth bypass | L8 | P0 | Yes | pytest / curl |
| TC-L8-003 | AMI IP restriction | L8 | P1 | Yes | pytest / curl |
| TC-L8-004 | SIP message fuzzing | L8 | P1 | Yes | SIPp / sipvicious |
| TC-L8-005 | SQL injection via API | L8 | P0 | Yes | pytest / curl |
| TC-L8-006 | Path traversal via API | L8 | P0 | Yes | pytest / curl |
| TC-L8-007 | Rate limiting on auth failures | L8 | P1 | Yes | SIPp / pytest |

---

## L4 -- Integration Tests (Telnyx PSTN)

These tests validate end-to-end PSTN call flow through the Telnyx SIP trunk.
They require a live Telnyx account with an active DID, a funded balance, and a
real mobile phone for two-way audio verification.

**Prerequisites common to all L4 tests:**

- RustPBX container running and healthy (L0 tests pass)
- Telnyx trunk loaded (`proxy.trunks.telnyx` in config)
- Telnyx routes loaded (`telnyx-inbound` and `telnyx-outbound`)
- Extension 1001 registered (via MicroSIP, PJSUA, or WebRTC client)
- Mobile phone available for PSTN call endpoints
- Telnyx account has sufficient balance for test calls

---

### TC-L4-001: Outbound PSTN Call via Telnyx

**Priority:** P0
**Automated:** Partial
**Tool:** PJSUA + mobile phone

**Preconditions:**

- Extension 1001 registered to RustPBX
- Telnyx outbound route active (`telnyx-outbound`, pattern `^\+?1?[2-9]\d{9}$`)
- Telnyx trunk configured with valid credentials (`sip.telnyx.com:5060`)
- Test mobile phone number known and reachable

**Steps:**

1. From the 1001 softphone (PJSUA or MicroSIP), dial the test mobile number
   (e.g., `+1XXXXXXXXXX`).
2. Observe RustPBX logs for route matching: expect `telnyx-outbound` match.
3. Observe RustPBX logs for INVITE forwarded to `sip.telnyx.com`.
4. Verify the mobile phone rings.
5. Answer the mobile phone.
6. Speak from the softphone; verify audio is heard on the mobile phone.
7. Speak from the mobile phone; verify audio is heard on the softphone.
8. Hang up from the softphone.
9. Verify the mobile phone disconnects.

**Expected Result:**

- RustPBX matches the outbound route and forwards INVITE to Telnyx.
- Telnyx returns `100 Trying`, then `180 Ringing`, then `200 OK`.
- Two-way audio flows for the duration of the call.
- BYE is sent and acknowledged; both legs terminate cleanly.
- AMI `/ami/v1/dialogs` shows no residual dialog after hangup.

**Pass Criteria:**

- Mobile phone rings within 10 seconds of dialing.
- Audio is audible in both directions (no one-way audio).
- Call duration reported by AMI matches wall-clock time within 2 seconds.
- No ERROR lines in RustPBX logs during the call.

**Fail Action:**

- **Severity:** Critical (P0) -- outbound calling is a core function.
- Check Telnyx trunk credentials and SIP connectivity.
- Verify outbound route pattern matches the dialed number.
- Check NAT/firewall: `external_ip` must be set for RTP to flow.
- Review Telnyx Mission Control call events for error codes.

---

### TC-L4-002: Inbound PSTN Call from Telnyx

**Priority:** P0
**Automated:** Partial
**Tool:** mobile phone + PJSUA

**Preconditions:**

- Extension 1001 registered to RustPBX
- Telnyx inbound route active (`telnyx-inbound`, matches DID `17072833106`, rewrites to `1001`)
- RustPBX reachable from Telnyx (public IP `74.207.251.126`, UDP 5060 open)
- `external_ip` configured in `config.toml` if behind NAT
- RTP port range (`20000-30000`) open in firewall

**Steps:**

1. From the test mobile phone, dial the Telnyx DID: `+1 (707) 283-3106`.
2. Observe RustPBX logs for incoming INVITE from Telnyx IP range.
3. Observe RustPBX logs for route match to `telnyx-inbound`.
4. Observe RustPBX logs for INVITE forwarded to local extension 1001.
5. Verify the 1001 softphone rings.
6. Answer on the 1001 softphone.
7. Speak from the mobile phone; verify audio is heard on the softphone.
8. Speak from the softphone; verify audio is heard on the mobile phone.
9. Hang up from the mobile phone.
10. Verify the softphone disconnects.

**Expected Result:**

- RustPBX receives INVITE from Telnyx with `To: <sip:17072833106@74.207.251.126>`.
- Route `telnyx-inbound` matches; `to.user` rewritten to `1001`.
- Extension 1001 rings and answers successfully.
- Two-way audio flows for the duration of the call.
- BYE from Telnyx propagates to extension; both legs terminate cleanly.

**Pass Criteria:**

- Softphone rings within 8 seconds of dialing from mobile.
- Audio is audible in both directions.
- Caller ID on the softphone shows the mobile phone number (or Telnyx-provided caller ID).
- No ERROR lines in RustPBX logs during the call.

**Fail Action:**

- **Severity:** Critical (P0) -- inbound PSTN is required for production.
- Verify Telnyx SIP Connection points to `74.207.251.126:5060`.
- Check firewall rules: UDP 5060 and RTP range must be open to Telnyx IP ranges.
- Confirm `external_ip` is set correctly for SDP media addresses.
- Check Telnyx Mission Control for delivery failures or error codes.

---

### TC-L4-003: Outbound Call to Invalid Number

**Priority:** P1
**Automated:** Yes
**Tool:** PJSUA / pytest

**Preconditions:**

- Extension 1001 registered to RustPBX
- Telnyx outbound route active

**Steps:**

1. From the 1001 softphone, dial an invalid number: `+10000000000`.
2. Wait for SIP response from Telnyx (via RustPBX).
3. Observe RustPBX logs for the error response.
4. Verify the softphone receives an error indication (busy tone or failure).
5. Check CDR for the failed call attempt.

**Expected Result:**

- RustPBX forwards INVITE to Telnyx.
- Telnyx returns a 4xx or 5xx error (typically `404 Not Found`, `480 Temporarily Unavailable`, or `503 Service Unavailable`).
- RustPBX relays the error to the originating extension.
- The softphone stops ringing / shows call failure.
- CDR records the attempt with a failure status and the SIP error code.

**Pass Criteria:**

- Error response received within 30 seconds.
- No RustPBX crash or panic in logs.
- CDR entry exists with `status != "completed"` and includes the SIP error code.
- AMI dialogs are empty after the failure (no leaked dialog state).

**Fail Action:**

- **Severity:** High (P1) -- error handling must be robust.
- If RustPBX hangs or crashes, file a bug against the call module.
- If no CDR is created for failed calls, file a bug against the CDR pipeline.

---

### TC-L4-004: Telnyx Trunk Registration / Health

**Priority:** P0
**Automated:** Yes
**Tool:** pytest / curl

**Preconditions:**

- RustPBX container running
- Telnyx trunk configured in `config.toml` or `config/trunks/*.toml`
- Network connectivity to `sip.telnyx.com` (UDP 5060)

**Steps:**

1. Query AMI health endpoint:
   ```bash
   curl -s http://74.207.251.126:8080/ami/v1/health
   ```
2. Verify the response includes trunk status information.
3. Check RustPBX logs for trunk loading:
   ```bash
   docker logs rustpbx 2>&1 | grep -i trunk
   ```
4. Send a SIP OPTIONS ping to `sip.telnyx.com` from the RustPBX host to verify
   SIP-level connectivity:
   ```bash
   docker exec rustpbx sh -c "echo | nc -u -w2 sip.telnyx.com 5060"
   ```
5. Verify DNS resolution for `sip.telnyx.com` from within the container:
   ```bash
   docker exec rustpbx sh -c "nslookup sip.telnyx.com || getent hosts sip.telnyx.com"
   ```

**Expected Result:**

- AMI health returns `"status":"running"` with no trunk errors.
- Logs show `trunks reloaded total=1` (or more if multiple trunks).
- DNS resolution for `sip.telnyx.com` succeeds.
- UDP connectivity to `sip.telnyx.com:5060` is not blocked.

**Pass Criteria:**

- AMI health HTTP status is `200`.
- JSON response includes `"status":"running"`.
- Trunk `telnyx` appears in loaded trunks.
- DNS resolves to at least one IP address.

**Fail Action:**

- **Severity:** Critical (P0) -- trunk must be operational for any PSTN test.
- If trunk does not load: check TOML syntax, credential format, file path.
- If DNS fails: check container DNS configuration (`/etc/resolv.conf`).
- If UDP blocked: check VPS firewall (iptables/ufw), Linode firewall, Docker networking.

---

### TC-L4-005: Caller ID Presentation

**Priority:** P1
**Automated:** Partial
**Tool:** PJSUA + mobile phone

**Preconditions:**

- Outbound route `telnyx-outbound` includes rewrite: `"from.user" = "17072833106"`
- Inbound route `telnyx-inbound` active
- Extension 1001 registered
- Mobile phone available

**Steps:**

1. **Outbound Caller ID test:**
   a. From extension 1001, dial the test mobile number.
   b. On the mobile phone, observe the incoming caller ID before answering.
   c. Verify the caller ID displays `+1 (707) 283-3106` (or the configured DID).
   d. Hang up.

2. **Inbound Caller ID test:**
   a. From the mobile phone, dial the Telnyx DID `+1 (707) 283-3106`.
   b. On the 1001 softphone, observe the incoming caller ID before answering.
   c. Verify the caller ID displays the mobile phone number.
   d. Hang up.

**Expected Result:**

- Outbound: The mobile phone shows the Telnyx DID as the caller, not the extension
  number or the RustPBX server IP.
- Inbound: The softphone shows the mobile phone's actual number, not the Telnyx
  DID or a generic number.

**Pass Criteria:**

- Outbound caller ID matches the DID configured in `from.user` rewrite (`17072833106`).
- Inbound caller ID matches the actual calling mobile number.
- SDP `From` header in outbound INVITE contains the DID (verify in sipflow capture or logs).

**Fail Action:**

- **Severity:** High (P1) -- incorrect caller ID breaks compliance and user trust.
- If outbound CID is wrong: check route rewrite rules (`from.user`, `from.host`).
- If inbound CID is missing: check Telnyx SIP Connection settings for P-Asserted-Identity passthrough.
- Review SIP message captures in sipflow for header inspection.

---

### TC-L4-006: PSTN Call Recording

**Priority:** P1
**Automated:** Partial
**Tool:** PJSUA + pytest

**Preconditions:**

- Recording enabled (`[recording] enabled = true, auto_start = true`)
- sipflow configured (`[sipflow] type = "local"`)
- CDR directory exists (`./config/cdr`)
- Extension 1001 registered

**Steps:**

1. **Outbound PSTN recording:**
   a. From extension 1001, dial the test mobile number via Telnyx.
   b. Answer on the mobile phone.
   c. Speak clearly for 10 seconds from each side.
   d. Hang up.
   e. Wait 5 seconds for recording finalization.
   f. List recording files:
      ```bash
      docker exec rustpbx sh -c "find /app/config/cdr/ -name '*.wav' -newer /tmp/test_marker 2>/dev/null"
      ```
   g. Verify a new WAV file exists.

2. **Inbound PSTN recording:**
   a. From the mobile phone, call the DID.
   b. Answer on extension 1001.
   c. Speak clearly for 10 seconds from each side.
   d. Hang up.
   e. Repeat steps (e) through (g) above.

**Expected Result:**

- A WAV recording file is created for each PSTN call (outbound and inbound).
- Each recording file has a non-zero file size (> 10 KB for a 10-second call).
- The recording captures audio from both the extension and the PSTN party.

**Pass Criteria:**

- WAV file exists in `/app/config/cdr/` (or subdirectory) after each call.
- File size > 10 KB (10 seconds of audio at 8 kHz mono PCMU = ~80 KB minimum).
- File is a valid WAV (parseable by `ffprobe` or `file` command).

**Fail Action:**

- **Severity:** High (P1) -- call recording is a compliance requirement.
- If no file created: check `[recording]` config, verify `auto_start = true`.
- If file is zero bytes: check media proxy mode, RTP flow, codec negotiation.
- If file is corrupt: check for premature container restart during recording.

---

### TC-L4-007: PSTN Call CDR

**Priority:** P1
**Automated:** Yes
**Tool:** pytest / curl

**Preconditions:**

- At least one completed outbound PSTN call (TC-L4-001) and one inbound PSTN
  call (TC-L4-002) have been executed.
- CDR storage is configured (`[callrecord] type = "local"`)

**Steps:**

1. Query the console call records page or API:
   ```bash
   curl -s -b <session_cookie> http://74.207.251.126:8080/console/call-records
   ```
2. Identify the CDR entry for the outbound PSTN call.
3. Verify CDR fields for the outbound call:
   - `direction` = `outbound`
   - `from` contains extension `1001`
   - `to` contains the dialed mobile number
   - `trunk` = `telnyx`
   - `status` = `completed`
   - `duration` > 0
   - Timestamps (`start_time`, `end_time`) are present and plausible
4. Identify the CDR entry for the inbound PSTN call.
5. Verify CDR fields for the inbound call:
   - `direction` = `inbound`
   - `from` contains the mobile phone number
   - `to` contains the DID `17072833106` or `1001`
   - `status` = `completed`
   - `duration` > 0

**Expected Result:**

- CDR entries exist for both outbound and inbound PSTN calls.
- All mandatory fields are populated with correct values.
- Duration matches the actual call length within 2 seconds.
- Trunk name `telnyx` is associated with both PSTN CDR entries.

**Pass Criteria:**

- Outbound CDR: `direction == "outbound"`, `trunk == "telnyx"`, `duration > 0`.
- Inbound CDR: `direction == "inbound"`, `from` matches the mobile number.
- No CDR entries with `null` direction, missing timestamps, or zero duration
  for connected calls.

**Fail Action:**

- **Severity:** High (P1) -- accurate CDR is required for billing and reporting.
- If CDR missing: check `[callrecord]` config, verify database/filesystem access.
- If direction is wrong: check route direction classification logic.
- If trunk name missing: check CDR hook integration with trunk metadata.

---

## L5 -- Media Quality Tests

These tests validate the quality and correctness of media handling: recording
file format, audio content, codec negotiation, RTP streams, and transcription.

**Prerequisites common to all L5 tests:**

- RustPBX container running and healthy
- At least two extensions registered (1001, 1002) or one extension + PSTN
- Recording enabled with `auto_start = true`
- `ffprobe` available on the test host for WAV analysis
- `tcpdump` available for packet capture

---

### TC-L5-001: Recording File Format Validation

**Priority:** P1
**Automated:** Yes
**Tool:** pytest + ffprobe

**Preconditions:**

- A completed call recording exists (from TC-L4-006 or an internal call).
- `ffprobe` installed on the test host.

**Steps:**

1. Copy a recording file from the container:
   ```bash
   docker cp rustpbx:/app/config/cdr/<recording_file>.wav /tmp/test_recording.wav
   ```
2. Run `ffprobe` to analyze the file:
   ```bash
   ffprobe -v quiet -print_format json -show_format -show_streams /tmp/test_recording.wav
   ```
3. Parse the JSON output and verify:
   - Container format is `wav`
   - Codec is `pcm_s16le` (16-bit PCM) or `pcm_mulaw` (G.711 mu-law)
   - Sample rate is `8000` Hz (for PCMU/PCMA) or `16000` Hz (for G.722)
   - Channels is `1` (mono) or `2` (stereo, if mixed recording)
   - Duration > 0 seconds
4. Verify file size is consistent with the format:
   - 8 kHz mono 16-bit PCM: ~16 KB per second
   - 8 kHz mono mu-law: ~8 KB per second

**Expected Result:**

- File is a valid WAV container.
- Audio codec, sample rate, and channel count match the expected recording format.
- Duration reported by `ffprobe` matches the call duration within 2 seconds.
- File size is proportional to the duration (not truncated or padded).

**Pass Criteria:**

- `ffprobe` exits with code 0 (valid file).
- `format.format_name` contains `wav`.
- `streams[0].sample_rate` is `8000` or `16000`.
- `streams[0].channels` is `1` or `2`.
- `format.duration` is > `0.0` and < call duration + 5 seconds.
- File size > 1 KB.

**Fail Action:**

- **Severity:** High (P1) -- corrupt recordings are useless for review or transcription.
- If format is wrong: check recording codec configuration.
- If duration is zero: check that the media bridge was active during the call.
- If file is truncated: check for I/O errors in container logs.

---

### TC-L5-002: Recording Audio Content

**Priority:** P2
**Automated:** Manual
**Tool:** ffplay / Audacity / headphones

**Preconditions:**

- A completed call recording exists where both parties spoke distinct words.
- Audio playback capability on the test host.

**Steps:**

1. Copy the recording file from the container:
   ```bash
   docker cp rustpbx:/app/config/cdr/<recording_file>.wav /tmp/test_recording.wav
   ```
2. Play the recording:
   ```bash
   ffplay -nodisp /tmp/test_recording.wav
   ```
   Or open in Audacity for waveform visualization.
3. Listen for:
   - Party A's voice (extension side) -- should be clearly audible.
   - Party B's voice (PSTN or other extension) -- should be clearly audible.
   - No extended silence where speech was expected.
   - No excessive noise, echo, or distortion.
4. If stereo: verify left channel contains one party and right channel contains the other
   (or that mono contains both parties mixed).

**Expected Result:**

- Both parties' speech is present in the recording.
- Audio is intelligible (words can be understood).
- No one-way audio (only one party recorded while the other is silent).
- No severe artifacts: clipping, digital noise, or codec errors.

**Pass Criteria:**

- Human listener confirms both parties are audible.
- No silence longer than 3 seconds where speech was expected.
- Audio quality is sufficient for post-call review and transcription.

**Fail Action:**

- **Severity:** Medium (P2) -- degraded recordings reduce review and transcription accuracy.
- One-way audio in recording: check media proxy mode (`media_proxy = "auto"`), NAT configuration.
- Silence only: check that RTP was flowing during the call (see TC-L5-004).
- Distortion: check codec mismatch between call legs and recording format.

---

### TC-L5-003: Codec Negotiation Match

**Priority:** P1
**Automated:** Yes
**Tool:** SIPp / PJSUA

**Preconditions:**

- RustPBX container running
- SIPp or PJSUA available for controlled SDP offers

**Steps:**

1. **Test 1: Offer PCMU only.**
   Using SIPp or PJSUA, send an INVITE to RustPBX with an SDP offer containing
   only `PCMU` (payload type 0):
   ```
   m=audio <port> RTP/AVP 0
   a=rtpmap:0 PCMU/8000
   ```
   Verify the SDP answer from RustPBX includes `PCMU`.

2. **Test 2: Offer PCMA only.**
   Send an INVITE with SDP containing only `PCMA` (payload type 8):
   ```
   m=audio <port> RTP/AVP 8
   a=rtpmap:8 PCMA/8000
   ```
   Verify the SDP answer includes `PCMA`.

3. **Test 3: Offer multiple codecs.**
   Send an INVITE with SDP containing `PCMU`, `PCMA`, `G722`:
   ```
   m=audio <port> RTP/AVP 0 8 9
   a=rtpmap:0 PCMU/8000
   a=rtpmap:8 PCMA/8000
   a=rtpmap:9 G722/8000
   ```
   Verify the SDP answer selects a codec from the offered set.

4. **Test 4: Offer unsupported codec only.**
   Send an INVITE with SDP containing only an obscure codec (e.g., GSM, payload
   type 3) that is not in the trunk's codec list:
   ```
   m=audio <port> RTP/AVP 3
   a=rtpmap:3 GSM/8000
   ```
   Verify RustPBX either rejects the call (`488 Not Acceptable Here`) or
   negotiates a fallback.

**Expected Result:**

- When offered a supported codec, RustPBX answers with that codec.
- When offered multiple codecs, RustPBX selects one from the offered set
  (preferably the highest-priority match with the trunk codec list).
- When offered only unsupported codecs, RustPBX responds with `488` or includes
  a supported codec in a counter-offer.

**Pass Criteria:**

- Test 1: SDP answer contains `PCMU`.
- Test 2: SDP answer contains `PCMA`.
- Test 3: SDP answer contains one of `PCMU`, `PCMA`, or `G722`.
- Test 4: Call is rejected with `488` or RustPBX counter-offers a supported codec.
- No crashes or panics on any codec offer.

**Fail Action:**

- **Severity:** High (P1) -- codec mismatch causes no-audio calls.
- If wrong codec selected: review B2BUA SDP manipulation logic.
- If crash on unusual SDP: file a bug with the offending SDP body.

---

### TC-L5-004: RTP Stream Validation

**Priority:** P1
**Automated:** Yes
**Tool:** tcpdump + pytest

**Preconditions:**

- Two extensions registered (1001 and 1002) or one extension + PSTN
- `tcpdump` available on the RustPBX host or within the container
- Call duration will be at least 10 seconds

**Steps:**

1. Start a packet capture for RTP traffic:
   ```bash
   docker exec -d rustpbx sh -c "timeout 30 tcpdump -i any -n 'udp portrange 20000-30000' -c 500 -w /tmp/rtp_capture.pcap"
   ```
2. Initiate a call from 1001 to 1002 (or to PSTN).
3. Answer the call.
4. Allow the call to remain connected for at least 10 seconds with audio
   (play a tone or speak).
5. Hang up.
6. Copy the capture file:
   ```bash
   docker cp rustpbx:/tmp/rtp_capture.pcap /tmp/rtp_capture.pcap
   ```
7. Analyze the capture:
   ```bash
   tcpdump -r /tmp/rtp_capture.pcap -n | head -50
   ```
8. Verify:
   - RTP packets flow in both directions (source/destination IP pairs).
   - Packet interval is approximately 20ms (50 packets/sec for standard codecs).
   - Both inbound and outbound RTP streams have packets.

**Expected Result:**

- RTP packets are captured on the media port range (20000-30000).
- Packets flow bidirectionally (two distinct source IP / port pairs).
- Packet rate is approximately 50 pps per direction (20ms interval).
- No extended gaps (> 200ms) in either direction during the active call.

**Pass Criteria:**

- Total captured packets > 100 for a 10-second call (expect ~500 per direction).
- Both direction streams have packets (no one-way RTP).
- Packet timing is regular (no burst-then-silence pattern).

**Fail Action:**

- **Severity:** High (P1) -- missing RTP means no audio.
- One-way RTP: check NAT traversal, `external_ip`, symmetric RTP.
- No RTP at all: check media proxy configuration, SDP port negotiation.
- Irregular timing: check system load, CPU contention in container.

---

### TC-L5-005: Transcript Generation

**Priority:** P1
**Automated:** Yes
**Tool:** pytest / curl

**Preconditions:**

- `addon-transcript` enabled (`addons = ["transcript"]` in config)
- Groq Whisper API key configured (via environment variable or config)
- A completed call recording exists with audible speech
- Transcription wrapper script available

**Steps:**

1. Identify a recent call recording with known speech content:
   ```bash
   docker exec rustpbx sh -c "ls -lt /app/config/cdr/*.wav | head -1"
   ```
2. Trigger transcription for the recording (via the transcript addon mechanism --
   this may be automatic post-call or triggered via API).
3. Wait for transcription to complete (poll for up to 60 seconds).
4. Retrieve the transcript:
   ```bash
   curl -s -b <session_cookie> http://74.207.251.126:8080/console/call-records/<call_id>
   ```
   Or check for a `.json` transcript file alongside the WAV:
   ```bash
   docker exec rustpbx sh -c "ls /app/config/cdr/*.json 2>/dev/null"
   ```
5. Verify the transcript output:
   - Is valid JSON.
   - Contains a `text` field with non-empty string.
   - Text contains recognizable words from the conversation.

**Expected Result:**

- Transcription completes within 60 seconds of call end (for a short call).
- Output is valid JSON with a transcript text field.
- Transcript text is non-empty and contains words spoken during the call.
- If word-level timestamps are present, they are monotonically increasing.

**Pass Criteria:**

- Transcript JSON parses without error.
- `text` field length > 10 characters.
- At least one known spoken word appears in the transcript.
- No API errors in RustPBX logs related to the Groq/Whisper endpoint.

**Fail Action:**

- **Severity:** High (P1) -- transcription is a key feature for call review.
- If no transcript generated: check addon configuration, API key validity.
- If API error: check Groq API endpoint URL, rate limits, file size limits (25 MB max).
- If transcript is empty/garbage: check recording quality (TC-L5-002).

---

## L6 -- Load / Stress Tests

These tests validate system behavior under concurrent load and sustained operation.
SIPp is the primary tool for generating SIP traffic at scale.

**Prerequisites common to all L6 tests:**

- RustPBX container running with adequate resources (recommended: 2+ CPU, 2+ GB RAM)
- SIPp installed and configured with scenario files
- Network path clear between SIPp host and RustPBX (low latency, no packet loss)
- Monitoring in place: `docker stats`, RustPBX logs, AMI health endpoint

---

### TC-L6-001: Concurrent SIP Registrations

**Priority:** P1
**Automated:** Yes
**Tool:** SIPp

**Preconditions:**

- RustPBX running with `ensure_user = true` (auto-creates users on REGISTER)
- SIPp installed with a REGISTER scenario file
- 50 unique SIP user credentials prepared (or `ensure_user` allows any)

**Steps:**

1. Create a SIPp REGISTER scenario (`register.xml`) that sends REGISTER, handles
   `401 Unauthorized` challenge, and re-sends with credentials.
2. Prepare a CSV injection file with 50 user entries (usernames 2001-2050).
3. Run SIPp with 50 simultaneous registrations:
   ```bash
   sipp 74.207.251.126:5060 -sf register.xml -inf users.csv \
     -l 50 -m 50 -r 50 -t un -p 6000
   ```
4. Monitor SIPp statistics for success/failure counts.
5. After completion, verify all 50 registrations via AMI or logs.
6. Check RustPBX memory and CPU usage during the test:
   ```bash
   docker stats rustpbx --no-stream
   ```

**Expected Result:**

- All 50 REGISTER transactions complete successfully (200 OK after auth challenge).
- SIPp reports 0 failed calls, 50 successful calls.
- RustPBX does not crash, restart, or become unresponsive.
- Response time for each REGISTER < 1 second.

**Pass Criteria:**

- SIPp success rate: 100% (50/50 successful registrations).
- No `5xx` SIP responses from RustPBX.
- RustPBX container remains running throughout.
- AMI health endpoint responds during the test.
- Peak memory usage < 500 MB.

**Fail Action:**

- **Severity:** High (P1) -- registration scalability affects production capacity.
- If some registrations fail: check for thread pool exhaustion, database locks (SQLite).
- If container crashes: check for OOM kill (`docker inspect --format='{{.State.OOMKilled}}'`).
- If response time > 1s: profile database backend performance.

---

### TC-L6-002: Concurrent Calls

**Priority:** P1
**Automated:** Yes
**Tool:** SIPp

**Preconditions:**

- RustPBX running
- At least 20 extensions pre-registered (or `ensure_user = true`)
- SIPp installed with a UAC (caller) and UAS (callee) scenario
- Internal routing configured for the test extension range

**Steps:**

1. Start a SIPp UAS (callee) on a secondary port to answer calls:
   ```bash
   sipp -sn uas -p 6100 -mi <test_host_ip>
   ```
2. Configure RustPBX routing to send calls for extensions 3001-3010 to the SIPp
   UAS IP.
3. Run SIPp UAC (caller) with 10 simultaneous calls:
   ```bash
   sipp 74.207.251.126:5060 -sf call_scenario.xml -inf callers.csv \
     -l 10 -m 10 -r 5 -d 10000 -t un -p 6000
   ```
   (10 calls, 5 calls/sec ramp, 10-second duration each)
4. Monitor:
   - SIPp caller statistics (setup time, success/failure).
   - SIPp callee statistics (received INVITEs, 200 OKs sent).
   - RustPBX logs for errors.
   - AMI dialogs count during peak.
5. After all calls complete, verify AMI dialogs is empty.

**Expected Result:**

- All 10 calls set up successfully (INVITE -> 200 OK).
- All 10 calls maintain for the configured duration (10 seconds).
- All 10 calls terminate cleanly (BYE -> 200 OK).
- RustPBX handles the concurrent load without errors.

**Pass Criteria:**

- SIPp reports 100% call success rate (10/10).
- Average call setup time < 500ms.
- No `5xx` SIP responses.
- RustPBX container remains running.
- AMI dialogs returns empty array after all calls end.
- No RTP port exhaustion errors in logs.

**Fail Action:**

- **Severity:** High (P1) -- concurrent call handling is critical for production.
- If calls fail: check RTP port range availability, media bridge capacity.
- If setup time is high: check SIP transaction timer configuration.
- If calls drop mid-session: check for timer/keepalive issues.

---

### TC-L6-003: API Throughput

**Priority:** P2
**Automated:** Yes
**Tool:** pytest / curl

**Preconditions:**

- RustPBX container running
- AMI endpoint accessible at `http://74.207.251.126:8080/ami/v1/health`

**Steps:**

1. Write a script (or pytest test) that sends 100 sequential HTTP GET requests to
   the AMI health endpoint as fast as possible:
   ```python
   import requests
   import time

   url = "http://74.207.251.126:8080/ami/v1/health"
   results = []
   for i in range(100):
       start = time.time()
       r = requests.get(url)
       elapsed = (time.time() - start) * 1000  # ms
       results.append((r.status_code, elapsed))
   ```
2. Calculate statistics:
   - Count of successful responses (HTTP 200).
   - Mean response time.
   - 95th percentile response time.
   - Maximum response time.
3. Repeat with 10 concurrent threads (using `concurrent.futures.ThreadPoolExecutor`)
   sending 10 requests each (100 total concurrent).

**Expected Result:**

- All 100 sequential requests return HTTP 200.
- All 100 concurrent requests return HTTP 200.
- No requests timeout or return 5xx errors.
- Response times remain consistent under load.

**Pass Criteria:**

- Sequential: 100% success rate, mean response time < 100ms, max < 500ms.
- Concurrent: 100% success rate, mean response time < 200ms, 95th percentile < 500ms.
- No HTTP 503 (service overloaded) responses.
- RustPBX container CPU usage < 80% during the test.

**Fail Action:**

- **Severity:** Medium (P2) -- API slowness degrades console and integration performance.
- If timeouts: check Tokio runtime thread count, HTTP connection limits.
- If 503s: check for request queue overflow in the HTTP server.
- If high latency: profile the health endpoint handler for blocking operations.

---

### TC-L6-004: Registration Storm Recovery

**Priority:** P2
**Automated:** Yes
**Tool:** SIPp

**Preconditions:**

- RustPBX running
- SIPp installed

**Steps:**

1. Blast 200 REGISTER requests within 1 second using SIPp:
   ```bash
   sipp 74.207.251.126:5060 -sf register.xml -inf users_200.csv \
     -l 200 -m 200 -r 200 -t un -p 6000
   ```
2. Immediately after the blast, monitor RustPBX:
   - Check if the container is still running: `docker ps`.
   - Check AMI health: `curl -s http://74.207.251.126:8080/ami/v1/health`.
   - Check for error spikes in logs: `docker logs rustpbx --tail 50`.
3. Wait 30 seconds for the system to stabilize.
4. Send a single normal REGISTER request and verify it succeeds:
   ```bash
   sipp 74.207.251.126:5060 -sf register.xml -inf single_user.csv \
     -l 1 -m 1 -r 1 -t un -p 6000
   ```
5. Verify the system has recovered to normal operation.

**Expected Result:**

- RustPBX may reject some of the 200 burst requests (acceptable under overload).
- The container does not crash or restart.
- AMI health endpoint remains responsive during and after the storm.
- The system recovers within 30 seconds and accepts new registrations normally.

**Pass Criteria:**

- Container is still running after the storm (`docker ps` shows `Up`).
- AMI health returns 200 within 5 seconds of the storm ending.
- Post-storm registration succeeds (single user registers successfully).
- No OOM kill or segfault in container logs.
- Recovery time < 30 seconds.

**Fail Action:**

- **Severity:** Medium (P2) -- storms happen in production (DDoS, misconfigured clients).
- If crash: check for unbounded queue growth, memory allocation failures.
- If slow recovery: check for transaction timer accumulation, connection table cleanup.
- If permanent hang: check for deadlock in SIP transaction layer.

---

### TC-L6-005: Long Duration Call

**Priority:** P2
**Automated:** Partial
**Tool:** PJSUA + pytest

**Preconditions:**

- Two extensions registered (1001, 1002) or one extension + PSTN
- PJSUA configured with auto-answer for the callee
- Recording enabled

**Steps:**

1. Set up the callee (1002) with auto-answer:
   ```bash
   pjsua --id sip:1002@74.207.251.126 --registrar sip:74.207.251.126 \
     --realm "*" --username 1002 --password test1002 \
     --auto-answer 200 --null-audio
   ```
2. Initiate a call from 1001 to 1002 using PJSUA or SIPp with a 30-minute duration.
3. Every 5 minutes during the call, verify:
   - AMI dialogs shows the active call.
   - RustPBX container is still running.
   - Memory usage is not growing unbounded.
4. At the 30-minute mark, hang up the call.
5. After hangup, verify:
   - CDR shows the call with duration of approximately 30 minutes.
   - Recording file exists and has appropriate size (~14 MB for 30 min at 8 kHz mono PCM).
   - AMI dialogs is empty (no leaked dialog state).

**Expected Result:**

- Call remains connected and stable for the full 30 minutes.
- No audio degradation over time.
- No memory leaks (container memory usage is stable, not monotonically increasing).
- CDR accurately reflects the 30-minute duration.
- Recording file is proportional in size to the duration.

**Pass Criteria:**

- Call survives 30 minutes without dropping.
- CDR duration is within 5 seconds of 30 minutes.
- Container memory growth < 50 MB over the 30-minute period.
- Recording file size is within 20% of the expected size for the duration.
- AMI dialogs empty after hangup.

**Fail Action:**

- **Severity:** Medium (P2) -- long calls are common in production (support calls, consultations).
- If call drops: check SIP session timer (re-INVITE/UPDATE keepalive), NAT timeout.
- If memory leak: profile Rust allocator, check for unbounded buffer growth in media path.
- If CDR duration is wrong: check timestamp capture at call start vs. call end.

---

## L7 -- Failover / Resilience Tests

These tests validate system behavior during failures and recovery scenarios.

**Prerequisites common to all L7 tests:**

- RustPBX running in Docker
- Docker CLI access on the host
- Ability to restart containers
- AMI health endpoint accessible

---

### TC-L7-001: Container Restart Recovery

**Priority:** P0
**Automated:** Yes
**Tool:** pytest / docker

**Preconditions:**

- RustPBX container running and healthy
- AMI health endpoint responding

**Steps:**

1. Record the current time.
2. Restart the container:
   ```bash
   docker restart rustpbx
   ```
3. Immediately start polling the SIP port and HTTP port:
   ```bash
   while ! curl -s --max-time 2 http://74.207.251.126:8080/ami/v1/health > /dev/null 2>&1; do
     sleep 1
   done
   ```
4. Record the time when the health endpoint first responds.
5. Calculate the recovery time (step 4 time - step 2 time).
6. Verify full functionality:
   - AMI health returns `"status":"running"`.
   - SIP port 5060 accepts a SIP OPTIONS request.
   - Console login page loads.
   - Trunks and routes are loaded (check logs for reload messages).

**Expected Result:**

- Container restarts and begins accepting connections.
- AMI health endpoint responds within 30 seconds of the restart command.
- All services (SIP, HTTP, WebSocket) are operational after recovery.
- Trunks and routes are reloaded from configuration files.

**Pass Criteria:**

- Recovery time (restart to healthy) < 30 seconds.
- AMI health returns `"status":"running"`.
- SIP OPTIONS to port 5060 returns `200 OK`.
- Logs show successful module loading: `acl, auth, presence, registrar, call`.
- Trunk `telnyx` appears in loaded trunks after restart.

**Fail Action:**

- **Severity:** Critical (P0) -- container restart is the primary recovery mechanism.
- If recovery > 30s: check for slow database migrations, DNS resolution delays.
- If services don't come up: check config file mounting, volume persistence.
- If trunk doesn't load: check trunk config file paths and permissions.

---

### TC-L7-002: Database Reconnection

**Priority:** P1
**Automated:** Yes
**Tool:** pytest / docker

**Preconditions:**

- RustPBX running with SQLite backend (`database_url = "sqlite://rustpbx.sqlite3"`)
- Note: For PostgreSQL testing, a separate postgres container is needed

**Steps:**

1. **SQLite test (simulated disruption):**
   a. Verify current database is accessible:
      ```bash
      docker exec rustpbx sh -c "ls -la /app/rustpbx.sqlite3"
      ```
   b. Make a test API call that writes to the database (e.g., create an extension
      via the console, or trigger a CDR write).
   c. Verify the write succeeded.

2. **PostgreSQL test (if applicable):**
   a. Verify RustPBX is connected to the PostgreSQL container:
      ```bash
      curl -s http://74.207.251.126:8080/ami/v1/health
      ```
   b. Stop the PostgreSQL container:
      ```bash
      docker stop postgres-test
      ```
   c. Attempt an API call that requires database access. Expect an error response.
   d. Restart the PostgreSQL container:
      ```bash
      docker start postgres-test
      ```
   e. Wait up to 30 seconds.
   f. Retry the API call. Verify it succeeds.

**Expected Result:**

- With SQLite: database operations work consistently (SQLite is in-process).
- With PostgreSQL: after the database container restarts, RustPBX reconnects
  automatically without requiring a RustPBX restart.
- No data corruption after reconnection.

**Pass Criteria:**

- PostgreSQL: RustPBX reconnects within 30 seconds of database availability.
- API calls succeed after reconnection.
- No panic or crash in RustPBX logs during database unavailability.
- CDR writes resume correctly after reconnection.

**Fail Action:**

- **Severity:** High (P1) -- database outages should not require full PBX restart.
- If no reconnection: check connection pool configuration (sea-orm / sqlx settings).
- If crash: file a bug -- database errors should be handled gracefully.
- If data corruption: check for incomplete transactions, WAL recovery.

---

### TC-L7-003: Trunk Failover (backup_dest)

**Priority:** P1
**Automated:** Yes
**Tool:** SIPp / pytest

**Preconditions:**

- RustPBX running with a trunk configured with `backup_dest`:
  ```toml
  [proxy.trunks.test_failover]
  dest = "sip:192.0.2.1:5060"          # unreachable (TEST-NET, RFC 5737)
  backup_dest = "sip:sip.telnyx.com:5060"  # real backup
  transport = "udp"
  username = "<telnyx_user>"
  password = "<telnyx_pass>"
  direction = "outbound"
  codec = ["PCMU"]
  ```
- A route pointing to the `test_failover` trunk for a specific dial pattern

**Steps:**

1. Configure the test trunk with an unreachable primary `dest` and the real
   Telnyx endpoint as `backup_dest`.
2. Reload trunks:
   ```bash
   curl -s -X POST http://74.207.251.126:8080/ami/v1/reload/trunks
   ```
3. From an extension, dial a number that matches the test trunk's route.
4. Observe RustPBX logs:
   - INVITE sent to primary `dest` (192.0.2.1) -- should timeout or fail.
   - After failure, INVITE sent to `backup_dest` (sip.telnyx.com).
5. Verify the call connects via the backup destination.
6. Verify CDR records the trunk name and possibly the failover event.

**Expected Result:**

- RustPBX attempts the primary destination first.
- After timeout or connection failure (408/503), RustPBX automatically tries `backup_dest`.
- The call connects successfully via the backup destination.
- The failover is transparent to the calling extension.

**Pass Criteria:**

- Call connects despite the primary destination being unreachable.
- Failover time < 10 seconds (from primary failure to backup attempt).
- Logs show both the failed primary attempt and the successful backup attempt.
- CDR records the call as completed (not failed).

**Fail Action:**

- **Severity:** High (P1) -- trunk failover is critical for PSTN reliability.
- If no failover attempted: check if `backup_dest` field is supported in current version.
- If failover too slow: check SIP transaction timeout timers (Timer B).
- If backup also fails: verify backup credentials and connectivity independently.

---

### TC-L7-004: Call Survival During Config Reload

**Priority:** P1
**Automated:** Partial
**Tool:** PJSUA + curl

**Preconditions:**

- Two extensions registered (1001, 1002)
- An active call between them (or one extension + PSTN)

**Steps:**

1. Establish a call between 1001 and 1002:
   ```bash
   # Using PJSUA for 1001, auto-answer on 1002
   ```
2. Verify the call is active:
   ```bash
   curl -s http://74.207.251.126:8080/ami/v1/dialogs
   ```
   Confirm at least one dialog is present.
3. While the call is active, trigger a route reload:
   ```bash
   curl -s -X POST http://74.207.251.126:8080/ami/v1/reload/routes
   ```
4. Verify the reload succeeds (HTTP 200 response).
5. Immediately verify the call is still active:
   ```bash
   curl -s http://74.207.251.126:8080/ami/v1/dialogs
   ```
6. Continue the call for 10 more seconds.
7. Verify audio still flows (speak and listen).
8. Hang up normally.
9. Verify CDR is created correctly.

**Expected Result:**

- Route reload completes successfully without affecting the active call.
- The call remains connected throughout the reload.
- Audio continues to flow in both directions during and after the reload.
- CDR shows the full call duration (including time during reload).

**Pass Criteria:**

- Call dialog persists in AMI before and after the reload.
- No SIP BYE or error responses generated by the reload.
- Audio is uninterrupted (no audible glitch or dropout).
- CDR duration covers the full call span.

**Fail Action:**

- **Severity:** High (P1) -- config reloads during business hours must not drop calls.
- If call drops: check if reload replaces the entire call routing table (should only update, not replace).
- If audio interruption: check if media bridge is affected by route reload.
- File a bug: hot reload must be non-disruptive.

---

### TC-L7-005: Graceful Shutdown (SIGTERM)

**Priority:** P0
**Automated:** Yes
**Tool:** pytest / docker

**Preconditions:**

- RustPBX container running
- Two extensions registered with an active call between them

**Steps:**

1. Establish a call between 1001 and 1002 (or SIPp UAC and UAS through RustPBX).
2. Verify the call is active via AMI dialogs.
3. Note the active call's dialog ID(s) and participant addresses.
4. Start a packet capture to observe SIP signaling:
   ```bash
   docker exec -d rustpbx sh -c "timeout 15 tcpdump -i any -n 'port 5060' -w /tmp/shutdown_capture.pcap"
   ```
5. Send SIGTERM to the RustPBX process:
   ```bash
   docker stop rustpbx --time=15
   ```
   (This sends SIGTERM, waits up to 15 seconds, then SIGKILL.)
6. Examine the packet capture for BYE messages:
   ```bash
   docker cp rustpbx:/tmp/shutdown_capture.pcap /tmp/ 2>/dev/null
   tcpdump -r /tmp/shutdown_capture.pcap -A 'port 5060' | grep -E "^(BYE|SIP/2.0)"
   ```
7. Alternatively, observe the SIPp or PJSUA client logs for incoming BYE.

**Expected Result:**

- Upon receiving SIGTERM, RustPBX sends BYE to all active call participants.
- Both call legs receive BYE and respond with `200 OK`.
- The container shuts down cleanly within the 15-second grace period.
- No SIGKILL is needed (process exits on its own after cleanup).

**Pass Criteria:**

- BYE messages observed in packet capture or client logs for all active calls.
- Container exit code is 0 (clean shutdown), not 137 (SIGKILL).
- No "zombie" calls left on SIPp or PJSUA clients (they receive disconnect).
- Shutdown completes within 15 seconds.

**Fail Action:**

- **Severity:** Critical (P0) -- ungraceful shutdown leaves phantom calls on trunks and clients.
- If no BYE sent: check SIGTERM handler in RustPBX; it may not implement graceful shutdown.
- If SIGKILL needed (exit code 137): the shutdown handler takes too long or hangs.
- File a bug: graceful shutdown with active call cleanup is a production requirement.

---

## L8 -- Security Tests

These tests validate that the system resists unauthorized access, malformed input,
and common attack vectors.

**Prerequisites common to all L8 tests:**

- RustPBX container running
- Test must not use production credentials for attack simulation
- All tests run from a controlled test environment
- Document any vulnerabilities found and report responsibly

---

### TC-L8-001: SIP Auth Bypass Attempt

**Priority:** P0
**Automated:** Yes
**Tool:** SIPp / sipvicious

**Preconditions:**

- RustPBX running with auth module enabled (`modules = ["acl", "auth", ...]`)
- SIPp or sipvicious installed on the test host

**Steps:**

1. **REGISTER without credentials:**
   Send a SIP REGISTER with no Authorization header:
   ```bash
   sipp 74.207.251.126:5060 -sf register_no_auth.xml -m 1 -t un
   ```
   The scenario sends only the initial REGISTER (no challenge response).

2. **REGISTER with wrong credentials:**
   Send a SIP REGISTER with invalid username/password:
   ```bash
   sipp 74.207.251.126:5060 -sf register.xml -inf bad_creds.csv -m 1 -t un
   ```
   Where `bad_creds.csv` contains `attacker;wrongpassword`.

3. **INVITE without registration:**
   Send a SIP INVITE from an unregistered endpoint:
   ```bash
   sipp 74.207.251.126:5060 -sf invite_no_reg.xml -m 1 -t un
   ```

4. **sipvicious scan (optional):**
   Run a SIP extension enumeration scan:
   ```bash
   svwar -m REGISTER -e 1000-1099 74.207.251.126
   ```

**Expected Result:**

- REGISTER without credentials: RustPBX responds with `401 Unauthorized` containing
  a `WWW-Authenticate` header with a challenge nonce.
- REGISTER with wrong credentials: RustPBX responds with `403 Forbidden` after the
  authentication challenge fails.
- INVITE without registration: RustPBX responds with `401 Unauthorized` or
  `403 Forbidden` (not `404`).
- sipvicious: unable to enumerate valid extensions from response differences.

**Pass Criteria:**

- No `200 OK` response to any unauthenticated or wrongly-authenticated request.
- Response codes are `401` or `403` (never `200`).
- No call leg is created for unauthenticated INVITEs.
- Extension enumeration does not reveal which extensions exist vs. do not exist
  (response codes should be consistent).

**Fail Action:**

- **Severity:** Critical (P0) -- auth bypass is a critical security vulnerability.
- If `200 OK` to unauthenticated REGISTER: check auth module loading order, ACL rules.
- If extension enumeration succeeds: check for response timing or code differences between
  valid and invalid extensions.
- File an immediate security bug.

---

### TC-L8-002: Console Auth Bypass

**Priority:** P0
**Automated:** Yes
**Tool:** pytest / curl

**Preconditions:**

- Console enabled (`[console] base_path = "/console"`)
- Admin user exists

**Steps:**

1. **Access protected page without session cookie:**
   ```bash
   curl -s -o /dev/null -w "%{http_code}" http://74.207.251.126:8080/console/
   ```
   Expected: `302` or `303` redirect to `/console/login`.

2. **Access protected API without session cookie:**
   ```bash
   curl -s -o /dev/null -w "%{http_code}" http://74.207.251.126:8080/console/extensions
   ```
   Expected: `302` redirect to login.

3. **Access with expired/invalid session cookie:**
   ```bash
   curl -s -o /dev/null -w "%{http_code}" -b "session=invalidtoken12345" \
     http://74.207.251.126:8080/console/
   ```
   Expected: `302` redirect to login.

4. **Access with forged session cookie:**
   ```bash
   curl -s -o /dev/null -w "%{http_code}" -b "session=aaaaaabbbbbbcccccc" \
     http://74.207.251.126:8080/console/extensions
   ```
   Expected: `302` redirect to login.

5. **Verify login page is accessible without auth:**
   ```bash
   curl -s -o /dev/null -w "%{http_code}" http://74.207.251.126:8080/console/login
   ```
   Expected: `200` (login page must be accessible).

**Expected Result:**

- All protected console routes redirect to login when accessed without a valid session.
- Invalid, expired, or forged session cookies are rejected.
- The login page itself is accessible without authentication.
- No console content is leaked in redirect responses.

**Pass Criteria:**

- Steps 1-4: HTTP response is `302` or `303` (redirect) to `/console/login`.
- Step 5: HTTP response is `200`.
- No console HTML content in the response body of unauthenticated requests
  (body should be empty or contain only the redirect).
- Response headers do not leak server version, internal paths, or debug info.

**Fail Action:**

- **Severity:** Critical (P0) -- console access controls are the primary admin defense.
- If `200` returned for unauthenticated request: session middleware is not applied to all routes.
- If content leaked: check for partial rendering before auth check.
- File an immediate security bug.

---

### TC-L8-003: AMI IP Restriction

**Priority:** P1
**Automated:** Yes
**Tool:** pytest / curl

**Preconditions:**

- AMI configured with IP restrictions. Current config has `allows = ["*"]` (open).
- For this test, temporarily change to a restrictive ACL:
  ```toml
  [ami]
  allows = ["127.0.0.1"]
  ```
- Reload configuration.

**Steps:**

1. **With restrictive ACL (`allows = ["127.0.0.1"]`):**
   a. From the RustPBX host (localhost), send an AMI request:
      ```bash
      curl -s http://127.0.0.1:8080/ami/v1/health
      ```
      Expected: `200 OK` with health JSON.
   b. From a different machine (or using a different source IP), send an AMI request:
      ```bash
      curl -s http://74.207.251.126:8080/ami/v1/health
      ```
      Expected: `403 Forbidden` or `401 Unauthorized`.

2. **With wildcard ACL (`allows = ["*"]`):**
   a. Restore the original configuration.
   b. From any IP, send an AMI request:
      ```bash
      curl -s http://74.207.251.126:8080/ami/v1/health
      ```
      Expected: `200 OK`.

**Expected Result:**

- When AMI allows is restricted, requests from non-allowed IPs are rejected.
- When AMI allows is `*`, all IPs are permitted.
- Rejected requests receive a clear error status (not a timeout).

**Pass Criteria:**

- Restrictive ACL: localhost request returns `200`, external request returns `403`.
- Wildcard ACL: all requests return `200`.
- No information leakage in the `403` response body (no stack traces, config details).

**Fail Action:**

- **Severity:** High (P1) -- AMI endpoints can reload config and terminate calls.
- If external request returns `200` with restrictive ACL: IP check is not implemented
  for AMI routes.
- If `allows` config is ignored: check AMI middleware initialization.
- File a security bug with priority.

---

### TC-L8-004: SIP Message Fuzzing

**Priority:** P1
**Automated:** Yes
**Tool:** SIPp / sipvicious

**Preconditions:**

- RustPBX running
- SIPp installed with custom fuzzing scenarios
- Container monitoring in place (detect crashes)

**Steps:**

1. **Oversized SIP message:**
   Send a SIP INVITE with an excessively large body (> 64 KB):
   ```bash
   sipp 74.207.251.126:5060 -sf fuzz_large_body.xml -m 1 -t un
   ```

2. **Malformed SIP headers:**
   Send a SIP message with:
   - Missing required headers (no `Via`, no `From`, no `Call-ID`).
   - Extremely long header values (> 8192 characters in a single header).
   - Invalid characters in SIP URIs (null bytes, control characters).
   - Negative `Content-Length`.

3. **Malformed SDP:**
   Send a SIP INVITE with garbage SDP body:
   ```
   v=0
   o=- 0 0 IN IP4 AAAAAAAAAAAAA
   m=audio 99999 RTP/AVP 999
   c=IN IP4 ../../etc/passwd
   ```

4. **Rapid malformed messages:**
   Send 100 malformed SIP messages in rapid succession.

5. After each fuzz test, verify:
   - Container is still running: `docker ps`.
   - AMI health responds: `curl -s http://74.207.251.126:8080/ami/v1/health`.
   - No segfault or panic in logs: `docker logs rustpbx --tail 20`.

**Expected Result:**

- RustPBX rejects or ignores all malformed messages.
- No crash, segfault, or panic occurs.
- The SIP stack remains operational for legitimate traffic after fuzzing.
- Error responses (if sent) are well-formed SIP responses.

**Pass Criteria:**

- Container survives all fuzz tests without restart.
- AMI health returns `200` after each fuzz round.
- No `SIGSEGV`, `panic`, or `unwrap failed` in container logs.
- A legitimate SIP REGISTER succeeds after fuzzing.

**Fail Action:**

- **Severity:** High (P1) -- SIP is exposed to the internet; fuzz resilience is critical.
- If crash on specific input: capture the exact message, file a security bug with reproduction.
- If memory growth: check for unbounded buffer allocation on malformed input.
- Report to upstream `rsipstack` project if the crash is in the SIP parsing layer.

---

### TC-L8-005: SQL Injection via API

**Priority:** P0
**Automated:** Yes
**Tool:** pytest / curl

**Preconditions:**

- Console login accessible
- Extension management accessible (after login)
- Database backend active (SQLite or PostgreSQL)

**Steps:**

1. **Login form SQL injection:**
   ```bash
   curl -s -X POST \
     -d "identifier=admin' OR '1'='1&password=anything" \
     http://74.207.251.126:8080/console/login \
     -w "\n%{http_code}"
   ```
   Expected: login fails (no auth bypass).

2. **Extension name injection:**
   After logging in with valid credentials:
   ```bash
   curl -s -X POST -b <session_cookie> \
     -d "username='; DROP TABLE extensions; --&password=test" \
     http://74.207.251.126:8080/console/extensions/create
   ```
   Expected: request rejected or safely handled.

3. **Search/filter injection:**
   ```bash
   curl -s -b <session_cookie> \
     "http://74.207.251.126:8080/console/call-records?search=' UNION SELECT * FROM users --"
   ```
   Expected: no data leakage from other tables.

4. **AMI endpoint injection:**
   ```bash
   curl -s "http://74.207.251.126:8080/ami/v1/hangup/'; DROP TABLE call_records; --"
   ```
   Expected: error response, no table dropped.

5. After all injection attempts, verify database integrity:
   ```bash
   docker exec rustpbx sh -c "sqlite3 /app/rustpbx.sqlite3 '.tables'"
   ```
   All expected tables must still exist.

**Expected Result:**

- No SQL injection succeeds.
- All injection attempts are safely handled (parameterized queries or input validation).
- No data from unauthorized tables is returned.
- Database schema is unmodified after all tests.

**Pass Criteria:**

- Login injection: HTTP response is `401` or `403`, not `303` (redirect to dashboard).
- Extension injection: request fails with validation error.
- Search injection: returns empty results or error, not data from other tables.
- All database tables still exist and contain their original data.
- No raw SQL error messages in any HTTP response body.

**Fail Action:**

- **Severity:** Critical (P0) -- SQL injection is a top-tier vulnerability.
- If any injection succeeds: file an immediate critical security bug.
- Check if sea-orm parameterized queries are used consistently.
- Audit all raw SQL queries in the codebase.

---

### TC-L8-006: Path Traversal via API

**Priority:** P0
**Automated:** Yes
**Tool:** pytest / curl

**Preconditions:**

- RustPBX running
- Recording download endpoint active
- Console accessible (after login)

**Steps:**

1. **Recording download path traversal:**
   ```bash
   curl -s -b <session_cookie> \
     "http://74.207.251.126:8080/console/recordings/../../etc/passwd" \
     -o /tmp/traversal_test.txt
   ```
   Verify the response does not contain `/etc/passwd` contents.

2. **Double-encoded path traversal:**
   ```bash
   curl -s -b <session_cookie> \
     "http://74.207.251.126:8080/console/recordings/%2e%2e%2f%2e%2e%2fetc%2fpasswd"
   ```

3. **Null byte injection:**
   ```bash
   curl -s -b <session_cookie> \
     "http://74.207.251.126:8080/console/recordings/legit.wav%00../../etc/passwd"
   ```

4. **Static file path traversal:**
   ```bash
   curl -s "http://74.207.251.126:8080/static/../config.toml"
   ```
   Verify the response does not contain configuration file contents.

5. **AMI endpoint path traversal:**
   ```bash
   curl -s "http://74.207.251.126:8080/ami/v1/../../config.toml"
   ```

**Expected Result:**

- No path traversal attempt returns files outside the expected directories.
- Responses are `404 Not Found`, `400 Bad Request`, or `403 Forbidden`.
- Configuration files, system files, and database files are never served.

**Pass Criteria:**

- No response body contains content from `/etc/passwd`, `config.toml`, or database files.
- All traversal attempts return `400`, `403`, or `404`.
- Response body does not contain file path or directory listing information.

**Fail Action:**

- **Severity:** Critical (P0) -- path traversal can expose credentials and configuration.
- If any file outside the allowed directory is served: file an immediate critical security bug.
- Check file-serving middleware for path canonicalization.
- Verify that the Rust HTTP framework (likely Axum/Actix) sanitizes paths by default.

---

### TC-L8-007: Rate Limiting on Auth Failures

**Priority:** P1
**Automated:** Yes
**Tool:** SIPp / pytest

**Preconditions:**

- RustPBX running with auth module
- SIPp or a script capable of rapid SIP REGISTER requests

**Steps:**

1. **SIP brute force test:**
   Send 50 REGISTER requests with incorrect passwords in rapid succession
   (within 10 seconds) for the same username:
   ```bash
   sipp 74.207.251.126:5060 -sf register.xml -inf brute_force.csv \
     -l 10 -m 50 -r 10 -t un
   ```
   Where `brute_force.csv` has 50 entries all with username `1001` and random
   wrong passwords.

2. After the 50 failed attempts, try a REGISTER with the correct password:
   ```bash
   sipp 74.207.251.126:5060 -sf register.xml -inf correct_creds.csv \
     -l 1 -m 1 -t un
   ```

3. **HTTP brute force test:**
   Send 50 login POST requests with wrong passwords:
   ```python
   for i in range(50):
       requests.post(
           "http://74.207.251.126:8080/console/login",
           data={"identifier": "admin", "password": f"wrong{i}"}
       )
   ```

4. After the 50 failed HTTP attempts, try logging in with correct credentials.

5. Record response codes and timing for all requests.

**Expected Result:**

- After a threshold of failed auth attempts (e.g., 10-20), the system begins
  rate limiting or temporarily blocking the source.
- Rate-limited responses may be `429 Too Many Requests` (HTTP) or `403` with
  delay (SIP).
- After a cooldown period, legitimate authentication succeeds.

**Pass Criteria:**

- **Ideal:** Rate limiting activates after N failed attempts (N < 20).
  Subsequent requests are rejected faster or with `429`/`403`.
- **Minimum acceptable:** Even without rate limiting, the system does not crash
  or become unresponsive under brute force.
- RustPBX remains operational for other users during the brute force.
- CPU usage does not spike above 90% during the attack.

**Fail Action:**

- **Severity:** High (P1) -- brute force resistance is a security baseline.
- If no rate limiting exists: document as a security gap, recommend implementation
  (e.g., fail2ban integration or built-in rate limiter).
- If system becomes unresponsive: check for auth processing bottlenecks,
  database lock contention during rapid auth lookups.
- Consider integrating with iptables/nftables for IP-level blocking.

---

## Appendix A: SIPp Scenario Templates

### A.1 REGISTER Scenario (register.xml)

```xml
<?xml version="1.0" encoding="UTF-8" ?>
<scenario name="REGISTER">
  <send>
    <![CDATA[
      REGISTER sip:[remote_ip] SIP/2.0
      Via: SIP/2.0/[transport] [local_ip]:[local_port];branch=[branch]
      From: <sip:[field0]@[remote_ip]>;tag=[pid]SIPpTag00[call_number]
      To: <sip:[field0]@[remote_ip]>
      Call-ID: [call_id]
      CSeq: 1 REGISTER
      Contact: <sip:[field0]@[local_ip]:[local_port]>
      Max-Forwards: 70
      Expires: 3600
      Content-Length: 0
    ]]>
  </send>

  <recv response="401" auth="true" />

  <send>
    <![CDATA[
      REGISTER sip:[remote_ip] SIP/2.0
      Via: SIP/2.0/[transport] [local_ip]:[local_port];branch=[branch]
      From: <sip:[field0]@[remote_ip]>;tag=[pid]SIPpTag00[call_number]
      To: <sip:[field0]@[remote_ip]>
      Call-ID: [call_id]
      CSeq: 2 REGISTER
      Contact: <sip:[field0]@[local_ip]:[local_port]>
      Max-Forwards: 70
      Expires: 3600
      [authentication username=[field0] password=[field1]]
      Content-Length: 0
    ]]>
  </send>

  <recv response="200" />
</scenario>
```

### A.2 CSV Injection File Format (users.csv)

```csv
SEQUENTIAL
1001;test1001
1002;test1002
2001;test2001
2002;test2002
```

### A.3 Call Scenario (call_scenario.xml)

```xml
<?xml version="1.0" encoding="UTF-8" ?>
<scenario name="UAC Call">
  <send retrans="500">
    <![CDATA[
      INVITE sip:[field1]@[remote_ip] SIP/2.0
      Via: SIP/2.0/[transport] [local_ip]:[local_port];branch=[branch]
      From: <sip:[field0]@[remote_ip]>;tag=[pid]SIPpTag00[call_number]
      To: <sip:[field1]@[remote_ip]>
      Call-ID: [call_id]
      CSeq: 1 INVITE
      Contact: <sip:[field0]@[local_ip]:[local_port]>
      Max-Forwards: 70
      Content-Type: application/sdp
      Content-Length: [len]

      v=0
      o=- 1234 1234 IN IP4 [local_ip]
      s=SIPp Call
      c=IN IP4 [local_ip]
      t=0 0
      m=audio [auto_media_port] RTP/AVP 0
      a=rtpmap:0 PCMU/8000
    ]]>
  </send>

  <recv response="100" optional="true" />
  <recv response="180" optional="true" />
  <recv response="200" />

  <send>
    <![CDATA[
      ACK sip:[field1]@[remote_ip] SIP/2.0
      Via: SIP/2.0/[transport] [local_ip]:[local_port];branch=[branch]
      From: <sip:[field0]@[remote_ip]>;tag=[pid]SIPpTag00[call_number]
      To: <sip:[field1]@[remote_ip]>[peer_tag_param]
      Call-ID: [call_id]
      CSeq: 1 ACK
      Max-Forwards: 70
      Content-Length: 0
    ]]>
  </send>

  <pause milliseconds="[field2]" />

  <send>
    <![CDATA[
      BYE sip:[field1]@[remote_ip] SIP/2.0
      Via: SIP/2.0/[transport] [local_ip]:[local_port];branch=[branch]
      From: <sip:[field0]@[remote_ip]>;tag=[pid]SIPpTag00[call_number]
      To: <sip:[field1]@[remote_ip]>[peer_tag_param]
      Call-ID: [call_id]
      CSeq: 2 BYE
      Max-Forwards: 70
      Content-Length: 0
    ]]>
  </send>

  <recv response="200" />
</scenario>
```

---

## Appendix B: Defect Severity Classification

| Severity | Definition | SLA |
|----------|-----------|-----|
| **P0 -- Critical** | System down, no workaround, security breach, data loss | Fix immediately |
| **P1 -- High** | Major feature broken, workaround exists but is unacceptable for production | Fix within 24 hours |
| **P2 -- Medium** | Feature degraded, workaround acceptable for short term | Fix within 1 week |
| **P3 -- Low** | Cosmetic, minor inconvenience, edge case | Fix in next release |

---

## Appendix C: Test Execution Checklist

| ID | Title | Run Date | Result | Tester | Notes |
|----|-------|----------|--------|--------|-------|
| TC-L4-001 | Outbound PSTN call via Telnyx | | | | |
| TC-L4-002 | Inbound PSTN call from Telnyx | | | | |
| TC-L4-003 | Outbound call to invalid number | | | | |
| TC-L4-004 | Telnyx trunk registration/health | | | | |
| TC-L4-005 | Caller ID presentation | | | | |
| TC-L4-006 | PSTN call recording | | | | |
| TC-L4-007 | PSTN call CDR | | | | |
| TC-L5-001 | Recording file format validation | | | | |
| TC-L5-002 | Recording audio content | | | | |
| TC-L5-003 | Codec negotiation match | | | | |
| TC-L5-004 | RTP stream validation | | | | |
| TC-L5-005 | Transcript generation | | | | |
| TC-L6-001 | Concurrent SIP registrations | | | | |
| TC-L6-002 | Concurrent calls | | | | |
| TC-L6-003 | API throughput | | | | |
| TC-L6-004 | Registration storm recovery | | | | |
| TC-L6-005 | Long duration call | | | | |
| TC-L7-001 | Container restart recovery | | | | |
| TC-L7-002 | Database reconnection | | | | |
| TC-L7-003 | Trunk failover (backup_dest) | | | | |
| TC-L7-004 | Call survival during config reload | | | | |
| TC-L7-005 | Graceful shutdown (SIGTERM) | | | | |
| TC-L8-001 | SIP auth bypass attempt | | | | |
| TC-L8-002 | Console auth bypass | | | | |
| TC-L8-003 | AMI IP restriction | | | | |
| TC-L8-004 | SIP message fuzzing | | | | |
| TC-L8-005 | SQL injection via API | | | | |
| TC-L8-006 | Path traversal via API | | | | |
| TC-L8-007 | Rate limiting on auth failures | | | | |

---

## Appendix D: Dependencies and Execution Order

```
L0 Smoke ──────► L1 Infrastructure ──────► L2 API Contract ──────► L3 SIP Functional
                                                                         │
                                                                         ▼
                                               ┌─────────────────────────┤
                                               │                         │
                                               ▼                         ▼
                                         L4 Integration            L5 Media Quality
                                         (Telnyx PSTN)             (Recording, Codec,
                                               │                    RTP, Transcript)
                                               │                         │
                                               └────────┬────────────────┘
                                                        │
                                                        ▼
                                                  L6 Load/Stress
                                                        │
                                                        ▼
                                               ┌────────┴────────┐
                                               │                 │
                                               ▼                 ▼
                                         L7 Failover       L8 Security
                                         / Resilience
```

**Key dependencies:**

- L4 requires L3 SIP Functional tests to pass (basic calling works before testing PSTN).
- L5 requires at least one successful call (L3 or L4) to have recording files.
- L6 requires L1 and L3 to pass (SIP registration and calling must work before load testing).
- L7 can run after L4 (needs calls and trunks to test failover).
- L8 can run after L1 (only needs basic connectivity, not full call flow).

---

*Document: `C:\Development\RustPBX\docs\tests\L4_L8_ADVANCED_TESTS.md`*
*Testing Plan Stage: 7 of 9*
*Last updated: 2026-02-21*
