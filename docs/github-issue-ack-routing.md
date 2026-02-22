# GitHub Issue: ACK to 200 OK uses wrong Request-URI with Record-Route loose routing

**Submit to:** https://github.com/restsend/rsipstack/issues/new (or restsend/rustpbx)

**Title:** ACK to 200 OK uses wrong Request-URI when Record-Route has lr (loose routing) parameter

---

## Description

When RustPBX (via rsipstack) sends an ACK for a 200 OK response to INVITE, the Request-URI is set to the first Record-Route entry's host instead of the Contact header from the 200 OK. This violates RFC 3261 Section 12.2.1.1 for loose routing and causes the remote party to never receive the ACK.

## Observed Behavior

200 OK from Telnyx contains:
```
Contact: <sip:15163987718@10.231.144.35:5070;transport=udp>
Record-Route: <sip:10.255.0.1;r2=on;lr;ftag=NymusCwo>
Record-Route: <sip:192.76.120.10;r2=on;lr;ftag=NymusCwo>
```

RustPBX sends ACK with:
```
ACK sip:15163987718@192.76.120.10:5060 SIP/2.0
Route: <sip:192.76.120.10;r2=on;lr;ftag=NymusCwo>
Route: <sip:10.255.0.1;r2=on;lr;ftag=NymusCwo>
```

**Problem:** The Request-URI `sip:15163987718@192.76.120.10:5060` uses the first Route entry's host, not the Contact from the 200 OK.

## Expected Behavior (per RFC 3261 Section 12.2.1.1)

When the first Route header has the `lr` parameter (loose routing), the ACK should use the Contact URI as the Request-URI:

```
ACK sip:15163987718@10.231.144.35:5070;transport=udp SIP/2.0
Route: <sip:192.76.120.10;r2=on;lr;ftag=NymusCwo>
Route: <sip:10.255.0.1;r2=on;lr;ftag=NymusCwo>
```

The message should still be sent to the first Route entry's address (192.76.120.10:5060), but the Request-URI must be the remote target (Contact).

## Impact

- Telnyx never receives the ACK
- Telnyx retransmits 200 OK every ~4 seconds for 32 seconds
- Telnyx eventually sends `BYE` with `Reason: SIP;cause=408;text="ACK Timeout"`
- This disrupts call stability and may cause audio quality issues
- Same behavior observed both on Linode VPS (direct public IP, no NAT) and behind NAT

## Environment

- rustpbx v0.3.18
- rsipstack v0.4.8
- SIP trunk: Telnyx (UDP transport)
- Server: Linode VPS with direct public IP (no NAT/SIP ALG)
