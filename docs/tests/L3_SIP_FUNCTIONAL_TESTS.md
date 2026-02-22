# L3 SIP Functional Tests

**Level:** L3 -- SIP Call Flow and Feature Testing
**Scope:** End-to-end SIP call flows through the B2BUA, including basic calls,
hold/resume, transfers, call forwarding, voicemail, DTMF, codec negotiation,
recording, CDR generation, and abnormal termination handling.
**Created:** 2026-02-21
**RustPBX Version:** 0.3.18
**SIP Proxy:** `127.0.0.1:5060` (UDP/TCP)
**HTTP API:** `http://localhost:8080`

---

## Summary Table

| ID | Title | Priority | Tool | Automated |
|----|-------|----------|------|-----------|
| TC-L3-001 | Internal call between two registered extensions | P0 | SIPp | Yes |
| TC-L3-002 | Call hold and resume | P1 | SIPp / PJSUA | Yes |
| TC-L3-003 | Blind transfer (REFER) | P1 | SIPp / PJSUA | Yes |
| TC-L3-004 | Attended transfer | P1 | PJSUA | Partial |
| TC-L3-005 | Call forward on no answer | P1 | SIPp | Yes |
| TC-L3-006 | Call forward on busy | P1 | SIPp | Yes |
| TC-L3-007 | Voicemail routing (no answer timeout) | P2 | SIPp | Partial |
| TC-L3-008 | Simultaneous ring (multi-device) | P2 | SIPp + PJSUA | Partial |
| TC-L3-009 | DTMF relay (RFC 2833) | P1 | SIPp | Yes |
| TC-L3-010 | Codec negotiation | P1 | SIPp | Yes |
| TC-L3-011 | Call recording triggers and file creation | P0 | SIPp + pytest | Yes |
| TC-L3-012 | CDR generation on call completion | P0 | SIPp + pytest | Yes |
| TC-L3-013 | Graceful BYE from caller and callee | P0 | SIPp | Yes |
| TC-L3-014 | Abnormal termination handling | P1 | SIPp | Yes |

---

## Prerequisites (All Tests)

- RustPBX Docker container running and healthy.
- SIP proxy listening on `0.0.0.0:5060` (UDP and TCP).
- HTTP API available at `http://localhost:8080`.
- Memory-backend test users registered: `1001`/`test1001`, `1002`/`test1002`.
- Internal route configured matching extensions `100x` with action `local`:
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
- SIPp installed and accessible in the test environment (version 3.6+).
- PJSUA (PJSIP command-line client) installed for interactive call tests.
- Recording enabled: `recording.enabled = true`, `recording.auto_start = true`.
- CDR enabled: `callrecord.type = "local"`, `callrecord.root = "./config/cdr"`.
- RTP port range accessible between test clients and the RustPBX container.
- For tests requiring a third extension, create extension `1003`/`test1003` via
  the console or API before running those tests.

## Tool Reference

| Tool | Purpose | Typical Usage |
|------|---------|---------------|
| **SIPp** | Automated SIP scenario execution (INVITE, REGISTER, BYE, REFER, re-INVITE) | XML scenario files, scripted call flows |
| **PJSUA** | Interactive SIP client with full call control (hold, transfer, conference) | CLI-driven, real-time audio, media negotiation |
| **pytest + requests** | HTTP API verification (CDR, recording file checks) | Post-call assertions via AMI/console API |
| **curl** | Quick HTTP checks during and after calls | AMI health, dialog verification |

---

## Test Cases

### TC-L3-001: Internal Call Between Two Registered Extensions

**Priority:** P0
**Automated:** Yes
**Tool:** SIPp

**Preconditions:**
- Extensions 1001 and 1002 are registered with the proxy.
- Internal route for `100x` is active.
- No active calls on either extension.

**Steps:**
1. **Register caller (UAC)** -- Use SIPp to register extension 1001 with the proxy:
   - Send `REGISTER sip:127.0.0.1` with `From: <sip:1001@127.0.0.1>`.
   - Receive `401 Unauthorized` with WWW-Authenticate challenge.
   - Re-send REGISTER with Authorization header (digest auth, password `test1001`).
   - Receive `200 OK` with Contact header and Expires value.
2. **Register callee (UAS)** -- Use a second SIPp instance to register extension 1002 similarly.
3. **INVITE** -- From the 1001 UAC, send `INVITE sip:1002@127.0.0.1`:
   - Include SDP offer with `m=audio <port> RTP/AVP 0` (PCMU), `a=rtpmap:0 PCMU/8000`.
   - Verify proxy challenges with `407 Proxy Authentication Required`.
   - Re-send INVITE with Proxy-Authorization header.
4. **100 Trying** -- Verify the proxy responds with `100 Trying` (provisional response).
5. **180 Ringing** -- Verify the UAS (1002) receives the INVITE and the proxy forwards `180 Ringing` to the UAC.
6. **200 OK** -- The UAS sends `200 OK` with SDP answer. Verify:
   - The proxy forwards `200 OK` to the UAC.
   - The SDP answer is present and contains a valid `m=audio` line.
   - The `Contact` header is rewritten by the B2BUA (not the callee's direct address).
7. **ACK** -- The UAC sends ACK. Verify the proxy forwards ACK to the UAS.
8. **RTP flow** -- Both sides send RTP packets for 3-5 seconds. Verify:
   - RTP packets are relayed through the media proxy (if `media_proxy = "auto"`).
   - Both sides receive RTP packets (non-zero SSRC, incrementing sequence numbers).
9. **Active dialog** -- During the call, query `GET /ami/v1/dialogs`. Verify:
   - At least one active dialog is returned.
   - The dialog contains `call_id`, `from`, `to` fields matching the test call.
10. **BYE** -- The UAC sends BYE. Verify:
    - The proxy forwards BYE to the UAS.
    - The UAS responds with `200 OK` to the BYE.
    - The proxy forwards `200 OK` back to the UAC.
11. **Dialog cleared** -- After BYE/200, query `GET /ami/v1/dialogs`. Verify the dialog list is empty (or the test call's dialog is absent).

**Expected Result:**
- Complete INVITE-200-ACK-BYE call flow succeeds through the B2BUA.
- SDP is negotiated and media (RTP) flows bidirectionally.
- The AMI API reflects the call during and after the call.
- Both call legs are properly torn down after BYE.

**Pass Criteria:**
- SIPp UAC scenario completes with 0 failed calls.
- SIPp UAS scenario completes with 0 unexpected messages.
- AMI `/dialogs` shows the call during the media phase and is empty after BYE.
- Total call setup time (INVITE to 200 OK) is less than 5 seconds.

**Fail Action:**
- **Severity: Critical.** Basic internal calling is the core feature. If INVITE is rejected, file P0. If media does not flow, file P0. If BYE does not tear down the call, file P0. If call setup exceeds 5 seconds, file P1 performance defect.

---

### TC-L3-002: Call Hold (re-INVITE with sendonly SDP) and Resume

**Priority:** P1
**Automated:** Yes
**Tool:** SIPp / PJSUA

**Preconditions:**
- Extensions 1001 and 1002 registered.
- An active call established between 1001 and 1002 (TC-L3-001 flow completed through step 8).

**Steps:**
1. **Establish call** -- Complete the INVITE-200-ACK flow between 1001 and 1002. Confirm RTP flows bidirectionally.
2. **HOLD (re-INVITE with sendonly)** -- From 1001, send a re-INVITE with modified SDP:
   - Change `a=sendrecv` to `a=sendonly` on the `m=audio` line.
   - Optionally set `c=IN IP4 0.0.0.0` (RFC 3264 hold indication).
3. **200 OK (hold acknowledged)** -- Verify the proxy forwards the re-INVITE to 1002. The UAS (1002) responds with `200 OK` containing SDP with `a=recvonly` (matching the hold direction).
4. **ACK (hold)** -- The UAC sends ACK for the re-INVITE 200.
5. **Verify hold state** -- During hold:
   - 1001 should NOT receive RTP from 1002 (media is paused in the callee direction).
   - Optionally verify the AMI dialog state reflects the hold if the API exposes call state.
6. **RESUME (re-INVITE with sendrecv)** -- From 1001, send another re-INVITE with:
   - `a=sendrecv` (restoring bidirectional media).
   - Valid `c=IN IP4 <actual_ip>` (if it was zeroed during hold).
7. **200 OK (resume acknowledged)** -- Verify the proxy forwards the re-INVITE and receives `200 OK` with `a=sendrecv`.
8. **ACK (resume)** -- The UAC sends ACK.
9. **Verify resumed media** -- After resume, both sides exchange RTP again for 2-3 seconds. Verify bidirectional packet flow.
10. **BYE** -- Tear down the call normally.

**Expected Result:**
- The B2BUA correctly relays re-INVITE requests for hold and resume.
- SDP direction attributes are properly forwarded and negotiated.
- Media pauses during hold and resumes after re-INVITE with sendrecv.

**Pass Criteria:**
- re-INVITE for hold receives 200 OK within 3 seconds.
- re-INVITE for resume receives 200 OK within 3 seconds.
- No RTP received by the holding party during the hold period (or greatly reduced -- only comfort noise).
- RTP resumes after the resume re-INVITE.
- Call tears down cleanly after hold/resume cycle.

**Fail Action:**
- **Severity: High.** Hold/resume is a standard call feature. If re-INVITE is rejected by the B2BUA, file P0. If SDP direction attributes are not forwarded correctly, file P1. If media does not pause during hold, file P1. If the call drops during hold/resume, file P0.

---

### TC-L3-003: Blind Transfer (REFER)

**Priority:** P1
**Automated:** Yes
**Tool:** SIPp / PJSUA

**Preconditions:**
- Extensions 1001, 1002, and 1003 registered.
- 1003 created via console if not already present (username `1003`, password `test1003`).
- Active call established between 1001 (caller) and 1002 (callee).

**Steps:**
1. **Establish call** -- 1001 calls 1002, complete INVITE-200-ACK, confirm RTP flows.
2. **REFER from 1001** -- 1001 sends REFER to the proxy requesting blind transfer of the call to 1003:
   - `REFER sip:1002@127.0.0.1`
   - `Refer-To: <sip:1003@127.0.0.1>`
   - `Referred-By: <sip:1001@127.0.0.1>`
3. **202 Accepted** -- Verify the proxy responds with `202 Accepted` to the REFER.
4. **NOTIFY (100 Trying)** -- Verify the proxy sends a NOTIFY to 1001 with `SIP/2.0 100 Trying` in the body (transfer in progress).
5. **New INVITE to 1003** -- Verify the proxy (or 1002, depending on B2BUA behavior) sends a new `INVITE sip:1003@127.0.0.1`. The INVITE should contain `Referred-By` or `Replaces` headers as appropriate.
6. **1003 answers** -- 1003 (UAS) answers with 200 OK. RTP is established between 1002 and 1003.
7. **NOTIFY (200 OK)** -- Verify 1001 receives a NOTIFY with `SIP/2.0 200 OK` (transfer succeeded).
8. **BYE to 1001** -- The original call leg to 1001 is terminated (either 1001 sends BYE or the proxy does after transfer completion).
9. **Verify transferred call** -- 1002 and 1003 are now in an active call. Verify:
   - AMI `/dialogs` shows a dialog between 1002 and 1003 (or their B2BUA call-ids).
   - RTP flows between 1002 and 1003.
10. **Teardown** -- 1002 or 1003 sends BYE. Call ends cleanly.

**Expected Result:**
- Blind transfer via REFER completes successfully.
- The original caller (1001) is disconnected after the transfer.
- The callee (1002) is now connected to the transfer target (1003).
- NOTIFY messages inform the transferor (1001) of transfer progress.

**Pass Criteria:**
- REFER receives 202 Accepted.
- 1003 receives INVITE and answers.
- 1001 disconnects after transfer.
- Post-transfer call between 1002-1003 has bidirectional RTP.
- AMI dialogs reflect the new call state.

**Fail Action:**
- **Severity: High.** Call transfer is a core business feature for receptionist and agent workflows. If REFER is rejected, file P0. If the transfer creates a dangling leg (1001 stays connected), file P0. If 1003 never receives INVITE, file P0. If transfer completes but no media, file P1.

---

### TC-L3-004: Attended Transfer

**Priority:** P1
**Automated:** Partial
**Tool:** PJSUA

**Preconditions:**
- Extensions 1001, 1002, and 1003 registered.
- PJSUA instances configured for all three extensions (attended transfer requires interactive consultation).

**Steps:**
1. **Call A-B** -- 1001 (A) calls 1002 (B). Complete INVITE-200-ACK. Confirm RTP.
2. **A puts B on hold** -- 1001 sends re-INVITE with `a=sendonly` to hold the call with 1002. Confirm hold (per TC-L3-002).
3. **A calls C (consultation)** -- While B is on hold, 1001 initiates a new call to 1003 (C):
   - `INVITE sip:1003@127.0.0.1`
   - Completes INVITE-200-ACK.
   - 1001 and 1003 can now speak (consultation leg).
4. **Verify two calls** -- AMI `/dialogs` should show two active dialogs:
   - Dialog 1: 1001-1002 (on hold).
   - Dialog 2: 1001-1003 (active consultation).
5. **A transfers B to C** -- 1001 sends REFER to transfer the held call (1002) to the consultation call (1003):
   - `REFER sip:1002@127.0.0.1`
   - `Refer-To: <sip:1003@127.0.0.1>` with `Replaces` header containing the dialog identifiers of the 1001-1003 call.
6. **Transfer executes** -- Verify:
   - 1002 is connected to 1003 (INVITE with Replaces, or re-INVITE).
   - 1001 is disconnected from both calls.
7. **Verify final state** -- AMI `/dialogs` shows one active dialog: 1002-1003.
8. **RTP verification** -- 1002 and 1003 exchange RTP bidirectionally.
9. **Teardown** -- 1002 or 1003 sends BYE. Call ends.

**Expected Result:**
- Attended transfer completes: A consults with C, then connects B to C.
- The transferor (A) is disconnected from both calls after transfer.
- The held party (B) and the consultation party (C) are now connected.

**Pass Criteria:**
- Two simultaneous calls are maintained during the consultation phase.
- After REFER with Replaces, B and C are connected.
- A is disconnected from both legs.
- Post-transfer call has bidirectional RTP.

**Fail Action:**
- **Severity: High.** Attended transfer is critical for professional call handling. If the B2BUA does not support Replaces header, file P0 feature gap. If transfer drops one or both parties, file P0. If the consultation call cannot be established while the first is on hold, file P1. This test is marked "Partial" because PJSUA scripting for attended transfer may require manual intervention.

---

### TC-L3-005: Call Forward on No Answer

**Priority:** P1
**Automated:** Yes
**Tool:** SIPp

**Preconditions:**
- Extensions 1001 and 1002 registered.
- Extension 1003 registered as the forward target.
- Call forward on no answer configured for extension 1002:
  - Forward-to: 1003.
  - Timeout: 15 seconds (configurable ring timeout before forwarding).
- This configuration may need to be set via the console or config.toml (the exact mechanism depends on RustPBX's call forward implementation).

**Steps:**
1. **Configure forwarding** -- Via console or API, set extension 1002 to forward on no answer to 1003 after 15 seconds.
2. **Call 1002** -- From 1001, send `INVITE sip:1002@127.0.0.1`.
3. **Proxy authentication** -- Complete 407 challenge.
4. **180 Ringing to 1001** -- Verify the proxy sends 180 Ringing to 1001 (1002 is ringing).
5. **1002 does NOT answer** -- The SIPp UAS for 1002 is configured to NOT send 200 OK (simulating no answer). It may send 180 Ringing but never answers.
6. **Wait for timeout** -- After 15 seconds (the configured no-answer timeout), verify the proxy:
   - Cancels the INVITE to 1002 (sends CANCEL).
   - Initiates a new INVITE to 1003 (the forward target).
7. **1003 receives INVITE** -- Verify 1003's UAS receives the forwarded INVITE.
8. **1003 answers** -- 1003 sends 200 OK. Verify:
   - The proxy forwards 200 OK to 1001.
   - RTP is established between 1001 and 1003.
9. **Verify call** -- AMI `/dialogs` shows an active call between 1001 and 1003. 1002 is not part of the call.
10. **Teardown** -- BYE from either party. Call ends cleanly.
11. **CDR verification** -- Check call records. Verify the CDR reflects the forwarding (e.g., original destination 1002, actual destination 1003, or two CDR entries).

**Expected Result:**
- The call rings 1002 for the configured timeout, then forwards to 1003.
- 1001 experiences continuous ringing (or a brief interruption) during the forward.
- After forwarding, 1001 is connected to 1003.

**Pass Criteria:**
- 1002 rings for approximately 15 seconds (within +/- 2 seconds tolerance).
- 1003 receives INVITE after the timeout.
- 1001-1003 call has bidirectional RTP after answer.
- CDR records the forwarded call.

**Fail Action:**
- **Severity: High.** Call forwarding on no answer is essential for business continuity. If forwarding never triggers, file P0. If forwarding triggers too early or too late, file P1. If the forwarded call has no media, file P0. If this feature is not yet implemented in RustPBX, file as a P0 feature gap with a workaround recommendation.

---

### TC-L3-006: Call Forward on Busy

**Priority:** P1
**Automated:** Yes
**Tool:** SIPp

**Preconditions:**
- Extensions 1001, 1002, and 1003 registered.
- Call forward on busy configured for extension 1002:
  - Forward-to: 1003.
- 1002 is currently on an active call (busy state).

**Steps:**
1. **Configure busy forwarding** -- Via console or API, set extension 1002 to forward on busy to 1003.
2. **Establish existing call on 1002** -- Use a SIPp UAC to call 1002 from a different context (or use extension 1001 to call 1002 and have 1002 answer). This puts 1002 in a busy state.
3. **Second call to 1002** -- From a different SIPp instance (acting as another caller or using extension 1001 if the first call was from a different source), send `INVITE sip:1002@127.0.0.1`.
4. **Busy detection** -- Verify the proxy detects that 1002 is busy:
   - 1002 responds with `486 Busy Here`, OR
   - The proxy determines 1002 has an active call and applies the forward rule.
5. **Forward to 1003** -- Verify the proxy:
   - Sends a new `INVITE sip:1003@127.0.0.1`.
   - 1003 receives the INVITE.
6. **1003 answers** -- 1003 sends 200 OK. Verify:
   - The caller receives 200 OK via the proxy.
   - RTP flows between the second caller and 1003.
7. **Verify both calls** -- AMI `/dialogs` shows two active calls:
   - Original call involving 1002.
   - Forwarded call with 1003.
8. **Teardown** -- End both calls with BYE.

**Expected Result:**
- When 1002 is busy, inbound calls are forwarded to 1003.
- The original call on 1002 is unaffected by the forwarding.
- The forwarded call completes normally.

**Pass Criteria:**
- Second call to busy 1002 is forwarded to 1003 (not rejected with 486).
- 1003 answers and RTP flows with the second caller.
- Original call on 1002 remains active and uninterrupted.
- CDR records the forwarded call.

**Fail Action:**
- **Severity: High.** Call forward on busy prevents missed calls. If busy forwarding is not triggered, file P0. If the original call is disrupted, file P0. If the forwarded call has no media, file P0. If this feature is not yet implemented, file as a P0 feature gap.

---

### TC-L3-007: Voicemail Routing (No Answer Timeout)

**Priority:** P2
**Automated:** Partial
**Tool:** SIPp

**Preconditions:**
- Extension 1001 registered (caller).
- Extension 1002 registered but configured to NOT answer (simulating unavailability).
- Voicemail system configured in RustPBX (voicemail endpoint or route).
- A voicemail route exists that catches calls after a no-answer timeout and routes to a voicemail handler.
- The voicemail handler may be a SIP endpoint, an IVR flow, or an internal module.

**Steps:**
1. **Configure voicemail routing** -- Ensure a route or call-forward rule exists that sends unanswered calls for 1002 to a voicemail endpoint after a configured timeout (e.g., 20 seconds).
2. **Call 1002** -- From 1001, send `INVITE sip:1002@127.0.0.1`.
3. **180 Ringing** -- Verify 1001 receives 180 Ringing.
4. **1002 does NOT answer** -- Wait for the no-answer timeout.
5. **Voicemail triggers** -- Verify the proxy:
   - Cancels the INVITE to 1002 (or receives 480/408 from 1002).
   - Routes the call to the voicemail endpoint.
6. **Voicemail answers** -- The voicemail endpoint sends 200 OK. Verify:
   - The caller (1001) receives 200 OK.
   - Media is established (the caller hears a voicemail greeting or silence/tone).
7. **Leave message** -- The caller sends RTP (simulated audio) for 5-10 seconds, then sends BYE.
8. **Verify voicemail storage** -- Check the voicemail storage location for a new message file or database entry associated with extension 1002.
9. **CDR verification** -- Verify the CDR records the call with an indication that it went to voicemail (status may be `voicemail` or destination may be the voicemail endpoint).

**Expected Result:**
- Unanswered calls are routed to voicemail after the timeout.
- The caller hears a voicemail greeting (or at least media is established).
- A voicemail message is stored.

**Pass Criteria:**
- Call is forwarded to voicemail after the configured timeout.
- Media is established between caller and voicemail endpoint.
- Voicemail message file or record is created.
- CDR reflects the voicemail routing.

**Fail Action:**
- **Severity: Medium.** Voicemail is important but not blocking for initial deployment. If voicemail is not yet implemented in RustPBX, document as a P1 feature gap. If routing to voicemail fails, file P1. If voicemail answers but no recording is stored, file P1. This test is marked "Partial" because voicemail may require additional infrastructure (media server, greeting files) that may not be available in all test environments.

---

### TC-L3-008: Simultaneous Ring (Multi-Device)

**Priority:** P2
**Automated:** Partial
**Tool:** SIPp + PJSUA

**Preconditions:**
- Extension 1002 registered from TWO different clients simultaneously (e.g., SIPp UAS instance A at port 5070 and SIPp UAS instance B at port 5071, both registered as 1002 with different Contact addresses).
- Or, simultaneous ring configured to ring both 1002 and 1003 when 1002 is called.
- RustPBX proxy supports forking or simultaneous ring (multi-contact registration or ring group).

**Steps:**
1. **Register multiple contacts** -- Register extension 1002 from two different SIP endpoints (different IP:port combinations). Verify both registrations succeed (both receive 200 OK with their respective Contact URIs in the binding).
2. **Call 1002** -- From 1001, send `INVITE sip:1002@127.0.0.1`.
3. **Forked INVITE** -- Verify the proxy forks the INVITE to both registered contacts of 1002:
   - Both UAS instances receive INVITE.
   - Both send `180 Ringing`.
4. **1001 receives ringing** -- Verify 1001 receives at least one 180 Ringing.
5. **First device answers** -- UAS instance A (first device) sends `200 OK`. Verify:
   - The proxy forwards `200 OK` to 1001.
   - The proxy sends `CANCEL` to UAS instance B (the device that did NOT answer).
6. **CANCEL to non-answering device** -- Verify UAS instance B receives CANCEL and responds with `200 OK` to the CANCEL, followed by `487 Request Terminated`.
7. **Media established** -- RTP flows between 1001 and the answering device (UAS A).
8. **Verify single call** -- AMI `/dialogs` shows exactly one active dialog (not two).
9. **Teardown** -- BYE from either party.

**Expected Result:**
- The proxy forks the INVITE to all registered contacts.
- When one device answers, the others are cancelled.
- Only one call leg is established.

**Pass Criteria:**
- Both registered contacts receive INVITE.
- Non-answering contact receives CANCEL after the other answers.
- Exactly one active dialog in AMI after answer.
- Bidirectional RTP with the answering device.

**Fail Action:**
- **Severity: Medium.** Simultaneous ring is important for multi-device users. If forking is not supported, file P1 feature gap. If both legs remain active (no CANCEL), file P0 (ghost calls). If CANCEL race condition leaves a dangling session, file P0. This test is marked "Partial" because multi-contact registration behavior may vary by RustPBX version and configuration.

---

### TC-L3-009: DTMF Relay (RFC 2833)

**Priority:** P1
**Automated:** Yes
**Tool:** SIPp

**Preconditions:**
- Extensions 1001 and 1002 registered.
- Active call established between 1001 and 1002.
- SDP negotiation includes `telephone-event` payload (RFC 2833/4733):
  - `a=rtpmap:101 telephone-event/8000`
  - `a=fmtp:101 0-16`

**Steps:**
1. **Establish call with telephone-event** -- From 1001, send INVITE with SDP including:
   ```
   m=audio <port> RTP/AVP 0 101
   a=rtpmap:0 PCMU/8000
   a=rtpmap:101 telephone-event/8000
   a=fmtp:101 0-16
   ```
   Complete INVITE-200-ACK. Verify the SDP answer from 1002 also includes `telephone-event`.
2. **Verify SDP negotiation** -- Confirm both SDP offer and answer include `telephone-event` with the same or compatible payload type number.
3. **Send DTMF digits** -- From 1001, send RFC 2833 RTP events for digits `1`, `2`, `3`, `#`:
   - Each DTMF event consists of 3 RTP packets: start, continuation, end (with the `E` bit set).
   - Payload type matches the negotiated value (e.g., 101).
   - Event codes: `1`=1, `2`=2, `3`=3, `#`=11.
   - Duration: 160ms per digit, 80ms inter-digit gap.
4. **Verify DTMF relay** -- On the 1002 UAS side, capture incoming RTP. Verify:
   - RFC 2833 events are received (payload type 101 or as negotiated).
   - The event codes match what was sent (1, 2, 3, 11).
   - The events are not corrupted (correct event code, volume, duration).
5. **Bidirectional DTMF** -- From 1002, send DTMF digits `4`, `5`, `6`. Verify 1001 receives them.
6. **Rapid DTMF** -- Send digits `0`-`9` in rapid succession (40ms inter-digit gap). Verify all 10 digits are received in order.
7. **DTMF during active audio** -- While sending normal audio RTP, interleave DTMF events. Verify both audio and DTMF are relayed correctly.
8. **Teardown** -- BYE from either party.

**Expected Result:**
- RFC 2833 telephone-event is negotiated in SDP.
- DTMF events pass through the B2BUA media proxy without corruption.
- All digits are received in the correct order.
- DTMF works bidirectionally.

**Pass Criteria:**
- SDP answer includes `telephone-event`.
- All sent DTMF digits are received on the other side with correct event codes.
- No DTMF digit loss during rapid sending.
- Event payload type is consistent with SDP negotiation.

**Fail Action:**
- **Severity: High.** DTMF relay is essential for IVR interaction and transfer operations. If DTMF events are stripped by the B2BUA, file P0. If digits are reordered or lost, file P0. If the payload type is incorrectly mapped, file P1.

---

### TC-L3-010: Codec Negotiation (PCMU, PCMA, G.722, Opus)

**Priority:** P1
**Automated:** Yes
**Tool:** SIPp

**Preconditions:**
- Extensions 1001 and 1002 registered.
- RustPBX codec configuration allows PCMU, PCMA, G.722, and Opus.

**Steps:**
1. **Multi-codec offer** -- From 1001, send INVITE with SDP offering multiple codecs:
   ```
   m=audio <port> RTP/AVP 0 8 9 111
   a=rtpmap:0 PCMU/8000
   a=rtpmap:8 PCMA/8000
   a=rtpmap:9 G722/8000
   a=rtpmap:111 opus/48000/2
   a=fmtp:111 minptime=10;useinbandfec=1
   ```
2. **Verify SDP answer** -- The proxy forwards the INVITE to 1002. 1002's UAS sends 200 OK with SDP answer. Verify:
   - The answer selects one or more codecs from the offer.
   - The selected codec(s) appear in the `m=audio` line.
   - The `a=rtpmap` lines match the selected codecs.
3. **Codec priority** -- If the proxy enforces codec priority, verify the answer selects the highest-priority codec according to the proxy's configuration (PCMU is typically default).
4. **Single codec offer (PCMU only)** -- Send INVITE with only PCMU:
   ```
   m=audio <port> RTP/AVP 0
   a=rtpmap:0 PCMU/8000
   ```
   Verify the answer includes PCMU.
5. **Single codec offer (PCMA only)** -- Send INVITE with only PCMA. Verify PCMA is accepted.
6. **Single codec offer (G.722 only)** -- Send INVITE with only G.722. Verify G.722 is accepted (if supported by the proxy configuration).
7. **Unsupported codec only** -- Send INVITE with only a codec not supported by RustPBX (e.g., `a=rtpmap:97 iLBC/8000`). Verify:
   - The proxy responds with `488 Not Acceptable Here` or `606 Not Acceptable`, OR
   - The proxy removes the unsupported codec and the call fails due to no common codec.
8. **Transcoding verification** -- If 1001 offers only PCMU and 1002 answers with PCMA (or vice versa), verify the B2BUA transcodes between the codecs and media flows correctly.
9. **RTP payload type** -- During an active call, capture RTP and verify the payload type in RTP headers matches the negotiated codec.
10. **Teardown** -- BYE from either party.

**Expected Result:**
- Multi-codec offers are handled correctly.
- Codec selection follows configured priority.
- Unsupported codecs are rejected gracefully.
- Transcoding works when call legs use different codecs.

**Pass Criteria:**
- Steps 1-3: SDP answer contains valid codec selection.
- Steps 4-6: single-codec offers are accepted for supported codecs.
- Step 7: unsupported-only offer is rejected (not accepted with wrong codec).
- Step 8: transcoded calls have audible, correct-sounding media.
- Step 9: RTP payload types match SDP negotiation.

**Fail Action:**
- **Severity: High.** Codec negotiation errors cause silent calls or call failures. If multi-codec SDP is mishandled, file P0. If transcoding produces garbled audio, file P1. If unsupported codec is silently accepted, file P1 (will cause media issues).

---

### TC-L3-011: Call Recording Triggers and File Creation

**Priority:** P0
**Automated:** Yes
**Tool:** SIPp + pytest

**Preconditions:**
- Recording enabled: `recording.enabled = true`, `recording.auto_start = true`.
- CDR directory exists: `./config/cdr/` inside the container.
- Extensions 1001 and 1002 registered.
- No recordings from previous test runs (clean state, or note the existing file count).

**Steps:**
1. **Note initial recording count** -- Count existing files in the recording directory:
   ```bash
   docker exec rustpbx sh -c "find /app/config/cdr/ -name '*.wav' | wc -l"
   ```
   Record the count as `N`.
2. **Establish call** -- From 1001, call 1002. Complete INVITE-200-ACK.
3. **Exchange audio** -- Send RTP from both sides for at least 5 seconds. Use a recognizable pattern (e.g., constant tone or specific RTP payload) if possible.
4. **End call** -- Send BYE. Wait 2-3 seconds for recording finalization.
5. **Verify recording file created** -- Count recording files again:
   ```bash
   docker exec rustpbx sh -c "find /app/config/cdr/ -name '*.wav' | wc -l"
   ```
   Verify the count is greater than `N` (at least one new file).
6. **Identify new recording** -- Find the most recently modified WAV file:
   ```bash
   docker exec rustpbx sh -c "ls -lt /app/config/cdr/*.wav 2>/dev/null | head -1"
   ```
7. **Verify file size** -- The recording should have non-trivial size:
   - For a 5-second call at 8 kHz mono PCM16: expect approximately 80,000 bytes (16 KB/sec * 5 sec = 80 KB).
   - Minimum acceptable: > 5,000 bytes (reject empty/corrupt files).
8. **Verify WAV header** -- Copy the file out and verify:
   ```bash
   docker cp rustpbx:/app/config/cdr/<filename> /tmp/test_recording.wav
   ```
   - Bytes 0-3: `RIFF`
   - Bytes 8-11: `WAVE`
   - Audio format tag at offset 20: `0x0001` (PCM) or other valid format.
9. **Verify audio content** -- If tools are available, check that the WAV file has non-silence audio:
   - Compute RMS amplitude or peak level.
   - Verify it is above a minimum threshold (not all zeros).
10. **Second call recording** -- Make another call (1002 to 1001). Verify a second recording file is created.
11. **Recording per call** -- Verify each completed call produces exactly one recording file.

**Expected Result:**
- Every completed call with `auto_start = true` produces a WAV recording file.
- Recording files are valid WAV containers with audio content.
- Recording files are stored in the configured CDR directory.

**Pass Criteria:**
- New WAV file appears after each call.
- File size > 5,000 bytes for a 5-second call.
- Valid RIFF/WAVE header.
- Audio content is non-silent (RMS > threshold).
- One recording per completed call.

**Fail Action:**
- **Severity: Critical.** Call recording is a compliance and operational requirement. If no recording file is created, file P0. If the file is created but empty (0 bytes), file P0. If the WAV header is invalid, file P1. If audio is all silence, file P1 (media proxy issue).

---

### TC-L3-012: CDR Generation on Call Completion

**Priority:** P0
**Automated:** Yes
**Tool:** SIPp + pytest

**Preconditions:**
- CDR enabled: `callrecord.type = "local"`, `callrecord.root = "./config/cdr"`.
- Extensions 1001 and 1002 registered.
- Console session available for CDR verification via HTTP.
- Note the current CDR count before the test.

**Steps:**
1. **Note initial CDR count** -- Query `GET /console/call-records` (or the underlying API) and count existing records. Record as `N`.
2. **Make a completed call** -- 1001 calls 1002:
   - INVITE-200-ACK.
   - Exchange RTP for 10 seconds.
   - BYE from 1001. Both sides confirm 200 OK to BYE.
3. **Wait for CDR processing** -- Wait 3-5 seconds for the async CDR pipeline to process.
4. **Verify new CDR** -- Query call records again. Verify:
   - Record count is `N + 1` (or a new record exists with a timestamp after the call start).
5. **Verify CDR fields** -- For the new CDR record, verify:
   - `call_id`: non-empty string, matches the SIP Call-ID from the INVITE (or the B2BUA's internal call-id).
   - `direction`: `internal` or the appropriate direction for an extension-to-extension call.
   - `from`: contains `1001` (the caller).
   - `to`: contains `1002` (the callee).
   - `duration`: approximately 10 seconds (within +/- 2 seconds tolerance).
   - `status`: `completed` (or equivalent success status).
   - `start_time` / `created_at`: timestamp within the last 60 seconds.
   - `end_time`: timestamp after `start_time`, difference approximately matches `duration`.
6. **Failed call CDR** -- From 1001, call an unregistered extension (`1009`). Verify:
   - The call fails (SIP 404 or 480).
   - A CDR record is still created with `status` = `failed`, `no-answer`, or an error status.
   - `duration` is 0 or near-zero.
7. **Cancelled call CDR** -- From 1001, call 1002 but send CANCEL before 1002 answers. Verify:
   - A CDR record is created with `status` = `cancelled` or `no-answer`.
   - `duration` is 0.
8. **CDR field consistency** -- Verify all CDR records have the same set of fields (no missing fields on some records).
9. **CDR ordering** -- Verify records are ordered by timestamp (most recent first or last, consistently).

**Expected Result:**
- Every call attempt (completed, failed, cancelled) generates a CDR record.
- CDR records contain accurate, complete field values.
- Duration and timestamps are within expected tolerance.

**Pass Criteria:**
- Completed call: CDR with `status=completed`, `duration` within 2 seconds of actual.
- Failed call: CDR with error status, `duration` = 0.
- Cancelled call: CDR with cancelled status, `duration` = 0.
- All required fields present and non-empty.
- Timestamps are accurate (within 5 seconds of wall clock).

**Fail Action:**
- **Severity: Critical.** CDR is essential for billing, compliance, and operational monitoring. If no CDR is generated, file P0. If CDR fields are missing or incorrect, file P0. If duration is wildly inaccurate, file P1. If failed/cancelled calls do not generate CDR, file P1.

---

### TC-L3-013: Graceful BYE from Caller and Callee

**Priority:** P0
**Automated:** Yes
**Tool:** SIPp

**Preconditions:**
- Extensions 1001 and 1002 registered.
- Two separate test scenarios (caller-initiated BYE and callee-initiated BYE).

**Steps:**

**Scenario A: Caller (1001) sends BYE**
1. Establish call: 1001 calls 1002, INVITE-200-ACK, RTP flows for 3 seconds.
2. **1001 sends BYE** -- UAC sends `BYE sip:1002@<proxy>`.
3. Verify the proxy forwards BYE to 1002.
4. 1002 responds with `200 OK` to the BYE.
5. Verify the proxy forwards `200 OK` back to 1001.
6. Verify both SIPp instances report the call as completed (no errors, no retransmissions stuck).
7. Verify AMI `/dialogs` is empty (dialog cleared).
8. Verify no RTP packets are sent after BYE/200 exchange.

**Scenario B: Callee (1002) sends BYE**
9. Establish a new call: 1001 calls 1002, INVITE-200-ACK, RTP flows for 3 seconds.
10. **1002 sends BYE** -- UAS sends `BYE sip:1001@<proxy>`.
11. Verify the proxy forwards BYE to 1001.
12. 1001 responds with `200 OK` to the BYE.
13. Verify the proxy forwards `200 OK` back to 1002.
14. Verify both SIPp instances report the call as completed.
15. Verify AMI `/dialogs` is empty.
16. Verify no RTP packets after BYE/200.

**Both scenarios:**
17. Verify CDR records are created for both calls (Scenario A and B).
18. Verify CDR `status` = `completed` for both.
19. Verify no orphaned dialogs or resource leaks by checking system health:
    ```bash
    curl -s http://localhost:8080/ami/v1/health
    ```

**Expected Result:**
- BYE from either party cleanly terminates the call through the B2BUA.
- The proxy forwards BYE and 200 OK in both directions.
- Dialogs are cleared, RTP stops, and CDR is generated.

**Pass Criteria:**
- Both scenarios complete without SIPp errors.
- AMI dialogs empty after each call.
- No BYE retransmissions (BYE/200 exchange completes on first attempt).
- CDR generated for both calls with `completed` status.
- System health is clean after both scenarios.

**Fail Action:**
- **Severity: Critical.** If BYE does not tear down the call, resources leak (ports, memory, file handles). If only caller-initiated BYE works but callee-initiated does not (or vice versa), file P0 with direction-specific details. If dialogs are not cleared, file P0.

---

### TC-L3-014: Abnormal Termination Handling

**Priority:** P1
**Automated:** Yes
**Tool:** SIPp

**Preconditions:**
- Extensions 1001 and 1002 registered.
- Active call established between 1001 and 1002.
- Container accessible for log inspection.

**Steps:**

**Scenario A: TCP connection reset during call (TCP transport)**
1. Establish a call between 1001 and 1002 using TCP transport (`INVITE sip:1002@127.0.0.1;transport=tcp`).
2. Confirm call is active (RTP flows, dialog in AMI).
3. **Abruptly close the TCP connection** -- On the 1001 side, force-close the TCP socket (RST, not graceful FIN). This can be done by killing the SIPp UAC process.
4. Verify the proxy detects the connection loss within a reasonable time (SIP transaction timeout, typically 32 seconds for INVITE transactions).
5. Verify the proxy sends BYE to 1002 to clean up the remaining call leg.
6. Verify 1002 receives BYE and responds with 200 OK.
7. Verify AMI `/dialogs` is eventually empty (within 60 seconds).
8. Verify CDR is generated with appropriate status (e.g., `failed`, `error`, or `completed` depending on how much audio was exchanged).

**Scenario B: UDP client disappears (no BYE, no response)**
9. Establish a call between 1001 and 1002 using UDP transport.
10. Confirm call is active.
11. **Kill the 1001 SIPp instance** -- Terminate without sending BYE.
12. Verify the proxy detects the absence via:
    - SIP session timer (if configured), OR
    - RTP timeout (no media received for N seconds), OR
    - Registration expiration.
13. Verify the proxy sends BYE to 1002 after detecting the timeout.
14. Verify AMI `/dialogs` is eventually empty.
15. Verify CDR is generated.

**Scenario C: Callee disappears mid-call**
16. Establish a call between 1001 and 1002.
17. **Kill the 1002 SIPp instance** mid-call.
18. Verify the proxy detects the callee's disappearance.
19. Verify the proxy sends BYE to 1001.
20. Verify AMI `/dialogs` is eventually empty.
21. Verify CDR is generated.

**Scenario D: SIP timeout during INVITE (no response from callee)**
22. From 1001, send INVITE to 1002. Configure 1002's UAS to NOT respond at all (no 100, no 180, no 200).
23. Verify the proxy retransmits the INVITE per SIP timer rules (Timer A/B for UDP).
24. After SIP transaction timeout (Timer B, ~32 seconds for UDP), verify:
    - The proxy responds to 1001 with `408 Request Timeout` or `504 Server Timeout`.
    - The INVITE transaction is cleaned up.
    - AMI shows no lingering dialog.

**Expected Result:**
- The B2BUA gracefully handles abnormal terminations.
- Remaining call legs are cleaned up (BYE sent to surviving party).
- No orphaned dialogs or resource leaks.
- CDR records all call attempts, including those that end abnormally.

**Pass Criteria:**
- Scenarios A-C: surviving party receives BYE within 60 seconds of the failure.
- All scenarios: AMI `/dialogs` empty within 90 seconds.
- All scenarios: CDR record generated for each call attempt.
- Scenario D: caller receives 408/504 within 35 seconds.
- System health check passes after all scenarios.
- No crash or restart of the RustPBX container during any scenario.

**Fail Action:**
- **Severity: High.** Abnormal termination handling prevents resource leaks and ghost calls. If the proxy never cleans up after a client disappears, file P0 (resource leak leads to eventual system failure). If BYE is never sent to the surviving party, file P0 (ghost call). If CDR is not generated, file P1. If the proxy crashes on TCP RST, file P0.

---

## Appendix A: SIPp Scenario File Templates

### Basic UAC (Caller) Scenario

```xml
<?xml version="1.0" encoding="ISO-8859-1" ?>
<scenario name="Basic UAC - Internal Call">
  <!-- REGISTER -->
  <send>
    <![CDATA[
      REGISTER sip:[remote_ip] SIP/2.0
      Via: SIP/2.0/[transport] [local_ip]:[local_port];branch=[branch]
      From: <sip:[field0]@[remote_ip]>;tag=[call_number]
      To: <sip:[field0]@[remote_ip]>
      Call-ID: reg-[call_id]
      CSeq: 1 REGISTER
      Contact: <sip:[field0]@[local_ip]:[local_port]>
      Expires: 3600
      Content-Length: 0
    ]]>
  </send>
  <recv response="401" auth="true" />
  <send>
    <![CDATA[
      REGISTER sip:[remote_ip] SIP/2.0
      Via: SIP/2.0/[transport] [local_ip]:[local_port];branch=[branch]
      From: <sip:[field0]@[remote_ip]>;tag=[call_number]
      To: <sip:[field0]@[remote_ip]>
      Call-ID: reg-[call_id]
      CSeq: 2 REGISTER
      Contact: <sip:[field0]@[local_ip]:[local_port]>
      Expires: 3600
      [authentication username=[field0] password=[field1]]
      Content-Length: 0
    ]]>
  </send>
  <recv response="200" />

  <!-- INVITE -->
  <send>
    <![CDATA[
      INVITE sip:[field2]@[remote_ip] SIP/2.0
      Via: SIP/2.0/[transport] [local_ip]:[local_port];branch=[branch]
      From: <sip:[field0]@[remote_ip]>;tag=[call_number]
      To: <sip:[field2]@[remote_ip]>
      Call-ID: [call_id]
      CSeq: 1 INVITE
      Contact: <sip:[field0]@[local_ip]:[local_port]>
      Content-Type: application/sdp
      Content-Length: [len]

      v=0
      o=- 1 1 IN IP4 [local_ip]
      s=SIPp Call
      c=IN IP4 [local_ip]
      t=0 0
      m=audio [auto_media_port] RTP/AVP 0
      a=rtpmap:0 PCMU/8000
    ]]>
  </send>
  <recv response="407" auth="true" />
  <send>
    <![CDATA[
      ACK sip:[field2]@[remote_ip] SIP/2.0
      Via: SIP/2.0/[transport] [local_ip]:[local_port];branch=[branch]
      From: <sip:[field0]@[remote_ip]>;tag=[call_number]
      To: <sip:[field2]@[remote_ip]>[peer_tag_param]
      Call-ID: [call_id]
      CSeq: 1 ACK
      Content-Length: 0
    ]]>
  </send>
  <!-- Re-INVITE with auth -->
  <send>
    <![CDATA[
      INVITE sip:[field2]@[remote_ip] SIP/2.0
      Via: SIP/2.0/[transport] [local_ip]:[local_port];branch=[branch]
      From: <sip:[field0]@[remote_ip]>;tag=[call_number]
      To: <sip:[field2]@[remote_ip]>
      Call-ID: [call_id]
      CSeq: 2 INVITE
      Contact: <sip:[field0]@[local_ip]:[local_port]>
      [authentication username=[field0] password=[field1]]
      Content-Type: application/sdp
      Content-Length: [len]

      v=0
      o=- 1 1 IN IP4 [local_ip]
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
      ACK sip:[next_url] SIP/2.0
      Via: SIP/2.0/[transport] [local_ip]:[local_port];branch=[branch]
      From: <sip:[field0]@[remote_ip]>;tag=[call_number]
      To: <sip:[field2]@[remote_ip]>[peer_tag_param]
      Call-ID: [call_id]
      CSeq: 2 ACK
      Content-Length: 0
    ]]>
  </send>

  <!-- Media phase -->
  <pause milliseconds="5000" />

  <!-- BYE -->
  <send>
    <![CDATA[
      BYE sip:[next_url] SIP/2.0
      Via: SIP/2.0/[transport] [local_ip]:[local_port];branch=[branch]
      From: <sip:[field0]@[remote_ip]>;tag=[call_number]
      To: <sip:[field2]@[remote_ip]>[peer_tag_param]
      Call-ID: [call_id]
      CSeq: 3 BYE
      Content-Length: 0
    ]]>
  </send>
  <recv response="200" />
</scenario>
```

### Basic UAS (Callee) Scenario

```xml
<?xml version="1.0" encoding="ISO-8859-1" ?>
<scenario name="Basic UAS - Answer and Wait for BYE">
  <!-- Wait for INVITE -->
  <recv request="INVITE" />
  <send>
    <![CDATA[
      SIP/2.0 180 Ringing
      [last_Via:]
      [last_From:]
      [last_To:];tag=[call_number]
      [last_Call-ID:]
      [last_CSeq:]
      Contact: <sip:[local_ip]:[local_port]>
      Content-Length: 0
    ]]>
  </send>
  <pause milliseconds="1000" />
  <send>
    <![CDATA[
      SIP/2.0 200 OK
      [last_Via:]
      [last_From:]
      [last_To:];tag=[call_number]
      [last_Call-ID:]
      [last_CSeq:]
      Contact: <sip:[local_ip]:[local_port]>
      Content-Type: application/sdp
      Content-Length: [len]

      v=0
      o=- 1 1 IN IP4 [local_ip]
      s=SIPp UAS
      c=IN IP4 [local_ip]
      t=0 0
      m=audio [auto_media_port] RTP/AVP 0
      a=rtpmap:0 PCMU/8000
    ]]>
  </send>
  <recv request="ACK" />

  <!-- Wait for BYE -->
  <recv request="BYE" />
  <send>
    <![CDATA[
      SIP/2.0 200 OK
      [last_Via:]
      [last_From:]
      [last_To:]
      [last_Call-ID:]
      [last_CSeq:]
      Content-Length: 0
    ]]>
  </send>
</scenario>
```

## Appendix B: SIPp Execution Examples

```bash
# Register and call as UAC (caller)
sipp -sf uac_scenario.xml 127.0.0.1:5060 \
  -inf users.csv \
  -m 1 \
  -l 1 \
  -r 1 \
  -trace_msg \
  -trace_err

# Run UAS (callee) waiting for incoming calls
sipp -sf uas_scenario.xml 127.0.0.1:5060 \
  -p 5070 \
  -inf users.csv \
  -m 1 \
  -trace_msg \
  -trace_err

# users.csv format:
# SEQUENTIAL
# field0;field1;field2
# 1001;test1001;1002
```

## Appendix C: Test Data

| Item | Value |
|------|-------|
| SIP Proxy | `127.0.0.1:5060` (UDP/TCP) |
| HTTP API | `http://localhost:8080` |
| Extension 1001 | Username `1001`, Password `test1001` |
| Extension 1002 | Username `1002`, Password `test1002` |
| Extension 1003 | Username `1003`, Password `test1003` (create via console) |
| Internal route | Pattern `^(100[0-9])$`, Action `local` |
| Codec priority | PCMU (0), PCMA (8), G.722 (9), Opus (111) |
| DTMF payload type | 101 (telephone-event/8000) |
| Recording directory | `/app/config/cdr/` (inside container) |
| CDR storage | Local filesystem at `/app/config/cdr/` |
| SIP transaction timeout | ~32 seconds (Timer B, UDP) |
| Registration expiry | 60 seconds (`registrar_expires = 60`) |

## Appendix D: Dependency Map

```
TC-L3-001 (Basic Call)
  ├── TC-L3-002 (Hold/Resume) -- requires active call
  ├── TC-L3-003 (Blind Transfer) -- requires active call + 3rd extension
  ├── TC-L3-004 (Attended Transfer) -- requires active call + 3rd extension
  ├── TC-L3-009 (DTMF) -- requires active call with media
  ├── TC-L3-010 (Codec Negotiation) -- requires INVITE/SDP flow
  ├── TC-L3-011 (Recording) -- requires completed call
  ├── TC-L3-012 (CDR) -- requires completed call
  └── TC-L3-013 (Graceful BYE) -- requires active call

TC-L3-005 (Forward No Answer) -- independent (uses ring timeout)
TC-L3-006 (Forward Busy) -- independent (uses busy detection)
TC-L3-007 (Voicemail) -- independent (uses no-answer timeout)
TC-L3-008 (Simultaneous Ring) -- independent (uses multi-registration)
TC-L3-014 (Abnormal Termination) -- requires active call (run last)
```

---

*Document: `C:\Development\RustPBX\docs\tests\L3_SIP_FUNCTIONAL_TESTS.md`*
*Last updated: 2026-02-21*
