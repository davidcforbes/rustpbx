# RustPBX Geographic Redundancy Design

## Document Metadata

| Field | Value |
|-------|-------|
| Task | rpbx-mwi.5 |
| Status | Draft |
| Created | 2026-02-24 |
| Depends On | rpbx-mwi.1 (SIP LB), rpbx-mwi.2 (shared state), rpbx-mwi.4 (failover) |

---

## 1. Deployment Models

Geographic redundancy extends the single-region clustering architecture described
in `clustering-architecture.md` to span multiple physical locations. Two
fundamental deployment models exist.

### 1.1 Active-Passive Across Regions

In active-passive, one region (primary) handles all traffic. The secondary region
runs idle, receiving replicated data but processing no calls.

| Aspect | Characteristic |
|--------|---------------|
| Capacity utilization | ~50% (standby region is wasted) |
| Failover trigger | Manual or automated DNS cutover |
| Failover time | 30s-5min depending on DNS TTL and detection speed |
| Data consistency | Simple: one-way replication from primary to secondary |
| Complexity | Low: no cross-region signaling during normal operation |
| Trunk registration | Single region only; re-register from secondary on failover |

### 1.2 Active-Active Across Regions

In active-active, both regions serve traffic simultaneously. Users are routed to
the nearest region by GeoDNS or anycast. Cross-region coordination is required
for shared state.

| Aspect | Characteristic |
|--------|---------------|
| Capacity utilization | ~100% (both regions serve calls) |
| Failover trigger | GeoDNS health checks remove unhealthy region |
| Failover time | DNS TTL (30-60s) + client retry |
| Data consistency | Requires multi-region replication with conflict resolution |
| Complexity | High: cross-region DB sync, split trunk registration, media routing |
| Trunk registration | Per-region or shared with SIP LB |

### 1.3 Recommendation: Active-Active with Region Affinity

Active-active is recommended for RustPBX because:

1. **No wasted capacity.** Both regions contribute to call handling, which matters
   for a VoIP platform where media processing is CPU- and bandwidth-intensive.
2. **Faster failover.** Users are already distributed. When a region fails, only
   that region's users must be re-routed, not the entire platform.
3. **Latency optimization.** Users connect to the nearest region, reducing both
   SIP signaling and RTP media latency.

Region affinity means that each user's SIP registration, call state, and media
path stay within a single region under normal conditions. Cross-region interaction
occurs only for:

- Inter-region calls (user in Region A calls user in Region B).
- Failover (Region A fails, its users re-register in Region B).
- Administrative queries (cluster-wide CDR search, global config changes).

This limits the blast radius of cross-region latency to cases where it is
unavoidable, while keeping intra-region operations fast and self-contained.

---

## 2. Database Replication

Different categories of data have different replication requirements. A single
replication strategy for all data would either over-provision latency guarantees
for non-critical data or under-provision them for critical data.

### 2.1 Data Classification

| Data Category | Latency Tolerance | Consistency Need | Volume |
|---------------|-------------------|-----------------|--------|
| SIP registrations | < 50ms write, < 10ms read | Strong within region, eventual across regions | Low (1 write per registration, every 60-300s per user) |
| Active call/dialog state | Per-node only (not replicated cross-region) | Per-node authoritative | N/A for cross-region |
| Configuration (trunks, routes, ACLs) | Seconds | Eventual | Very low (admin changes) |
| CDR / call records | Seconds to minutes | Eventual | Moderate (1 row per completed call) |
| Presence | < 1s | Eventual | Low |
| Recordings / media files | Minutes to hours | Eventual | High (bandwidth-dependent) |

### 2.2 SIP Registrations

Registrations are the most latency-sensitive shared state. A call to a user
requires looking up that user's registration to determine their contact address.

**Per-region registration store (recommended):**

Each region maintains its own registration database (PostgreSQL + Redis cache as
described in `clustering-architecture.md`). Registrations are region-local:
a user registered in Region A has their binding stored in Region A's database.

When a call from Region B targets a user in Region A, Region B's routing layer
must query Region A's registration store. This is handled by the inter-region
call routing path (Section 4), not by replicating all registrations everywhere.

**Why not replicate all registrations globally?**

- With 10,000 users refreshing every 60 seconds, that is ~167 writes/second
  globally. Replicating every write cross-region adds latency to every
  registration without benefiting intra-region calls (the 95%+ case).
- Stale registration data cross-region would cause misrouted calls. The freshness
  guarantee needed (< 1s) conflicts with typical cross-region replication lag.

**Failover exception:** When a region fails, its registrations become stale. Users
detect registration timeout (typically 30-120s) and re-register against the
surviving region via DNS failover. The surviving region's database receives the
new registrations directly. No cross-region replication is needed for this path.

### 2.3 CDR / Call Records

CDR data is written once when a call completes. It is not read in the real-time
call path. Asynchronous replication is acceptable.

**Strategy:** Each region writes CDRs to its local PostgreSQL instance.
Cross-region replication uses PostgreSQL logical replication (publish/subscribe)
or an application-level sync job to consolidate CDRs into a central analytics
database.

- Replication lag: up to 60 seconds acceptable.
- Conflict risk: none (CDRs are append-only with globally unique session IDs).
- For consolidated reporting, a read replica in a third location (or one of the
  two regions) aggregates CDRs from both regions.

### 2.4 Configuration Data

Configuration changes (trunk definitions, route tables, ACL rules) are
infrequent and admin-initiated. Eventual consistency with a propagation delay of
a few seconds is acceptable.

**Strategy:** Designate one PostgreSQL instance as the config primary (or use
multi-master with last-write-wins for non-conflicting config keys). When an
admin saves a config change, it writes to the primary database. PostgreSQL
streaming replication propagates it to the secondary. A Redis pub/sub event
(cross-region, via Redis Cluster or a message bridge) triggers config reload on
all nodes in both regions.

### 2.5 Database Technology Recommendations

**Option A: PostgreSQL with Streaming Replication**

Standard PostgreSQL with one primary and one async streaming replica per region.

```
Region A (primary)                Region B (replica)
+------------------+              +------------------+
| PostgreSQL       | ---async---> | PostgreSQL       |
| (read/write)     |  streaming   | (read-only)      |
+------------------+  replication +------------------+
```

Pros:
- Mature, well-understood. Existing SeaORM integration works unchanged.
- Async replication minimizes cross-region latency impact on writes.
- Promotion of replica to primary is a standard PostgreSQL operation.

Cons:
- Single write primary creates a regional dependency for config changes.
- Failover requires manual or scripted promotion of the replica.
- Read-only replica in Region B means registration writes must go to Region A
  (mitigated by per-region registration stores).

**Option B: CockroachDB for Multi-Region**

CockroachDB provides native multi-region support with configurable locality for
tables.

```
Region A                          Region B
+------------------+              +------------------+
| CockroachDB      | <--Raft---> | CockroachDB      |
| (read/write)     |  consensus  | (read/write)     |
+------------------+              +------------------+
```

Pros:
- True multi-master: both regions can write to all tables.
- Per-table locality pins data to a region (registrations stay local).
- Automatic failover with no manual promotion.
- SQL-compatible (works with SeaORM via the PostgreSQL wire protocol).

Cons:
- Adds operational complexity (CockroachDB cluster management).
- Write latency for globally-replicated tables includes cross-region RTT.
- Overkill for 2-region deployments where PostgreSQL streaming replication
  suffices.

**Recommendation:** Start with PostgreSQL streaming replication for its
simplicity and compatibility with the existing codebase. Evaluate CockroachDB
if the deployment grows beyond two regions or if true multi-master writes become
a requirement.

---

## 3. DNS-Based Geographic Routing

DNS is the primary mechanism for directing users to the nearest region. SIP
clients resolve the PBX domain into transport/host/port targets using the
standard RFC 3263 resolution chain.

### 3.1 GeoDNS for SIP (NAPTR/SRV Records)

GeoDNS services (AWS Route 53, Cloudflare, NS1) return different DNS responses
based on the querying resolver's geographic location.

**Example configuration (two regions: us-east, us-west):**

```
; NAPTR records (transport selection, same for all regions)
pbx.example.com.  300 IN NAPTR 10 10 "s" "SIP+D2U" "" _sip._udp.pbx.example.com.
pbx.example.com.  300 IN NAPTR 20 10 "s" "SIP+D2T" "" _sip._tcp.pbx.example.com.

; SRV records (GeoDNS returns region-specific targets)
; Resolver in US-East gets:
_sip._udp.pbx.example.com. 60 IN SRV 10 100 5060 east1.pbx.example.com.
_sip._udp.pbx.example.com. 60 IN SRV 20 100 5060 west1.pbx.example.com.

; Resolver in US-West gets:
_sip._udp.pbx.example.com. 60 IN SRV 10 100 5060 west1.pbx.example.com.
_sip._udp.pbx.example.com. 60 IN SRV 20 100 5060 east1.pbx.example.com.

; A records (static, not geo-dependent)
east1.pbx.example.com. 300 IN A 203.0.113.10
west1.pbx.example.com. 300 IN A 198.51.100.10
```

The SRV priority field controls preference: lower values are preferred. By giving
the local region priority 10 and the remote region priority 20, compliant SIP
clients will prefer the local region and fall back to the remote on failure.

**SRV TTL:** Set to 60 seconds for SRV records. This balances DNS query load
against failover speed. During a region outage, clients re-resolve after at most
60 seconds and discover the remote region as the only available target.

### 3.2 Anycast Considerations for SIP over UDP

IP anycast advertises the same IP address from multiple locations via BGP. The
network routes packets to the nearest announcing location.

**SIP over UDP with anycast is problematic:**

- UDP is connectionless. If BGP routes shift mid-dialog, subsequent SIP messages
  (re-INVITE, BYE) may arrive at a different region than the one that processed
  the INVITE.
- Anycast works well for stateless protocols (DNS) but poorly for stateful dialog
  protocols like SIP.

**SIP over TCP/TLS with anycast is safer:** TCP connections are pinned to a
specific server once established. BGP route changes do not affect existing
connections. New connections after a route change go to the new nearest location.

**Recommendation:** Do not use anycast for SIP over UDP. Use GeoDNS with SRV
records instead. If anycast is desired for simplicity, restrict it to SIP over
TCP/TLS or WebSocket transports where connection persistence prevents mid-dialog
routing issues.

### 3.3 HTTP/WebSocket Routing (WebRTC Clients)

WebRTC clients connect via HTTP/WebSocket to the RustPBX web interface. These
connections can be routed geographically using standard HTTP infrastructure.

**Options:**

1. **CDN/Global Load Balancer** (AWS ALB + Route 53, Cloudflare Load Balancing,
   GCP Global LB): Routes HTTP/WebSocket connections to the nearest healthy
   region. Health checks on the `/health/ready` endpoint ensure traffic is only
   sent to operational regions.

2. **GeoDNS for the web endpoint:** Same approach as SIP -- resolve
   `web.pbx.example.com` to the nearest region's IP via GeoDNS.

WebSocket connections are persistent (like TCP), so once established they remain
pinned to a region. This aligns well with the region-affinity model.

---

## 4. Media Server Placement

Media latency is the primary driver for geographic placement decisions. RTP
audio quality degrades noticeably beyond 150ms one-way latency. Cross-region
network paths typically add 30-80ms of one-way latency (US coast-to-coast), which
can push total mouth-to-ear delay beyond acceptable thresholds if media is
relayed through a distant region.

### 4.1 Media Servers Per Region

Each region runs its own RustPBX nodes with local media processing. The B2BUA
model (as described in `clustering-architecture.md` Section 4.1) means that each
RustPBX node relays RTP through itself. Media stays within the region for
intra-region calls.

```
Region A                          Region B
+--------+    RTP     +--------+  +--------+    RTP     +--------+
| Phone  | <-------> | RustPBX | | RustPBX | <-------> | Phone  |
| (user) |           | Node A  | | Node B  |           | (user) |
+--------+           +--------+  +--------+           +--------+
                          |            |
                          +--SIP only--+
                          (signaling for
                          inter-region calls)
```

For an intra-region call (both users in Region A), the entire call -- signaling
and media -- stays within Region A. Latency is determined only by the local
network.

### 4.2 Cross-Region Media Relay for Inter-Region Calls

When a user in Region A calls a user in Region B, there are two media routing
options.

**Option 1: Direct media between regions (recommended for quality)**

RustPBX in Region A terminates the caller's RTP leg. RustPBX in Region B
terminates the callee's RTP leg. The two RustPBX instances relay media to each
other over a direct inter-region link.

```
Phone A <--RTP--> RustPBX-A <==inter-region RTP==> RustPBX-B <--RTP--> Phone B
```

Latency: Phone A to Phone B = (A to RustPBX-A) + (A to B) + (RustPBX-B to B).
The inter-region hop is unavoidable but occurs only once.

**Option 2: Single-region relay (simpler, higher latency)**

The call is processed entirely by one region. The remote user's RTP traverses the
entire distance twice (to the processing region and back).

```
Phone A <--RTP--> RustPBX-A <--RTP (cross-region round-trip)--> Phone B
```

Latency: Phone B's media travels cross-region twice. For US east-to-west (40ms
one-way), this adds 80ms of media latency compared to Option 1.

**Recommendation:** Implement Option 1 (direct inter-region media relay) for
inter-region calls. This requires the RustPBX nodes in each region to establish
an RTP bridge between them, using the existing `MediaBridge` infrastructure with
a remote peer address in the other region.

### 4.3 TURN Server Placement Strategy

TURN servers relay media for clients behind restrictive NATs or firewalls that
block direct RTP. TURN server placement follows the same principle as media server
placement: minimize the distance between the TURN server and the client.

**Recommended deployment:**

| Component | Placement |
|-----------|-----------|
| TURN server | One per region, co-located with or near RustPBX nodes |
| STUN server | Same servers (TURN servers also handle STUN) |
| Client configuration | Provide region-specific TURN URIs via the WebRTC config endpoint |

```
Region A                           Region B
+--------+                         +--------+
| TURN-A | (for clients in A)      | TURN-B | (for clients in B)
+--------+                         +--------+
```

The RustPBX WebRTC configuration endpoint (`/api/webrtc/config` or equivalent)
should return TURN server URIs appropriate for the client's region. This can be
determined by the client's connecting IP address or by the region of the RustPBX
node serving the request (since GeoDNS already routes clients to the nearest
region).

---

## 5. Trunk Registration Per Region

SIP trunk providers (Telnyx, Twilio, etc.) require REGISTER messages to maintain
connectivity. In a multi-region deployment, trunk registration must be
coordinated to avoid conflicts and ensure inbound calls reach the correct region.

### 5.1 Option A: One Trunk Connection Per Region

Each region registers independently with the trunk provider using separate SIP
credentials or separate SIP connections.

```
Region A                          Trunk Provider
+----------+   REGISTER (cred A)  +----------+
| RustPBX-A| ------------------>  |          |
+----------+                      | Telnyx   |
                                  |          |
Region B                          |          |
+----------+   REGISTER (cred B)  |          |
| RustPBX-B| ------------------>  |          |
+----------+                      +----------+
```

**Inbound routing:** The trunk provider routes inbound calls to the region that
is closest to the caller (if the provider supports geographic routing) or to a
designated primary region.

Pros:
- Clean separation. Each region is self-contained for trunk connectivity.
- Failover is provider-managed: if Region A stops registering, the provider
  routes inbound to Region B.

Cons:
- Requires the trunk provider to support multiple registrations per DID or
  multiple SIP connections. Not all providers support this.
- May require additional SIP trunks (additional cost).
- DID assignment across regions must be managed carefully.

### 5.2 Option B: Single Trunk in Primary Region, Re-Route Inbound

Only the primary region registers with the trunk provider. Inbound calls land
in the primary region, which then routes them to the appropriate region based on
the callee's registration location.

```
                                  Trunk Provider
                                  +----------+
Region A (primary)                |          |
+----------+   REGISTER           | Telnyx   |
| RustPBX-A| <--> INVITE ------> |          |
+----------+                      +----------+
      |
      | (inter-region SIP if callee is in Region B)
      v
Region B
+----------+
| RustPBX-B|
+----------+
```

Pros:
- Simplest trunk configuration. Works with any provider.
- No multi-registration coordination needed.

Cons:
- Single point of failure for inbound calls. If the primary region fails,
  inbound calls fail until the trunk registration is moved to Region B.
- Inbound calls to Region B users incur cross-region SIP signaling latency
  (though media can still be direct if Option 1 from Section 4.2 is used).
- Trunk registration failover requires the distributed lock mechanism from
  `clustering-architecture.md` Section 5.3, extended across regions.

### 5.3 Option C: Shared Trunk with SIP Load Balancer

A SIP load balancer (Kamailio, OpenSIPS, or a cloud SIP LB) sits in front of
both regions and terminates the trunk connection. The SIP LB distributes inbound
calls to the nearest healthy region.

```
Trunk Provider
+----------+
|          |
| Telnyx   | --INVITE--> +----------+ ---> Region A (RustPBX-A)
|          |             |  SIP LB  |
|          | <-REGISTER- | (global) | ---> Region B (RustPBX-B)
+----------+             +----------+
```

Pros:
- Decouples trunk registration from individual regions.
- The SIP LB can perform geographic routing of inbound calls.
- Both regions are reachable for inbound calls without provider-side config.

Cons:
- The SIP LB itself must be geographically redundant (or it becomes a SPOF).
- Adds another component to operate and monitor.
- SIP LB must understand the registration state of both regions to make routing
  decisions.

### 5.4 Recommendation

**Start with Option B** (single trunk, primary region) for initial multi-region
deployments. It works with every trunk provider, requires minimal coordination,
and the existing `TrunkRegistrationModule` with Redis-based leader election
(from `clustering-architecture.md`) naturally extends to cross-region leader
election by sharing the same Redis cluster.

**Graduate to Option A** when the trunk provider supports it and when inbound
call volume justifies eliminating the primary-region dependency. Telnyx supports
multiple SIP connections per account, making this feasible.

**Consider Option C** only for deployments with three or more regions or when
working with trunk providers that do not support multi-registration.

---

## 6. Failover Across Regions

### 6.1 Region Health Monitoring

Each region's health is assessed by combining signals from multiple layers.

**Health signals:**

| Signal | Source | Frequency | Weight |
|--------|--------|-----------|--------|
| HTTP health check (`/health/ready`) | External monitor or GeoDNS provider | Every 10-30s | Primary |
| SIP OPTIONS ping | Cross-region peer or SIP LB | Every 10s | Primary |
| Database connectivity | Node self-check | Every 5s | Internal |
| Redis connectivity | Node self-check | Every 5s | Internal |
| Active call count trend | Prometheus metrics | Continuous | Advisory |

**Region health state machine:**

```
HEALTHY ---(2+ consecutive health check failures)---> DEGRADED
DEGRADED ---(all nodes in region failing)---> FAILED
FAILED ---(health checks passing again)---> RECOVERING
RECOVERING ---(stable for 60s)---> HEALTHY
```

GeoDNS providers (Route 53, Cloudflare) perform external health checks against
the `/health/ready` endpoint. When a region's health checks fail, the provider
automatically removes that region's DNS records from responses. New DNS queries
resolve only to the healthy region.

### 6.2 DNS Failover TTL Considerations

DNS TTL directly determines the maximum time between a region failure and clients
being redirected to the healthy region.

| TTL | Failover Time (worst case) | DNS Query Load | Recommendation |
|-----|---------------------------|----------------|----------------|
| 10s | ~20s (TTL + detection) | Very high | Too aggressive for SIP |
| 30s | ~60s | High but manageable | Good for critical deployments |
| 60s | ~90s | Moderate | **Recommended default** |
| 300s | ~330s | Low | Too slow for VoIP |

**Recommended:** 60-second TTL for SRV records, 30-second TTL if the deployment
requires sub-2-minute failover. NAPTR and A records can use longer TTLs (300s)
since they change less frequently.

**Caveat:** Many SIP phones and softphones cache DNS results beyond the TTL.
Aggressive caching by clients means that some portion of traffic will continue
to be directed at the failed region for minutes after DNS failover. The phones
will experience SIP timeout (32s for UDP, connection refused for TCP) before
retrying with a fresh DNS lookup.

### 6.3 Active Call Handling During Region Failure

Active calls are anchored to a region by their RTP media path and SIP dialog
state. When a region fails, all calls active in that region are lost.

**Behavior timeline:**

```
T+0s:     Region A fails (network, power, or process crash)
T+0-2s:   RTP packets stop flowing. Phones detect silence.
T+10-30s: GeoDNS health checks detect failure, begin removing Region A.
T+30-60s: DNS TTL expires. New DNS queries return only Region B.
T+30-90s: SIP session timers fire (if enabled) or phones detect RTP timeout.
           Phones hang up the dead call and display a call failure.
T+60-120s: Phones attempt re-REGISTER. DNS resolves to Region B.
            Phones register in Region B.
T+120s+:  New calls route through Region B. Service is restored.
```

**Key design decision:** No attempt is made to recover active calls across
regions. The latency and complexity of cross-region dialog reconstruction
(transferring SDP state, re-establishing RTP, re-INVITEing both parties) is
not justified given that:

- Cross-region RTP handoff would cause 2-10 seconds of silence minimum.
- The reconstructed call would have different media characteristics (jitter
  buffer reset, possible codec renegotiation).
- Users generally accept that a network outage drops a call and are willing to
  re-dial.

**Mitigation:** Enable SIP session timers (`session_expires = 120`) so that
phones detect failed calls within 2 minutes rather than waiting for RTP timeout
(which may be 30 seconds to indefinite depending on the phone's implementation).

### 6.4 Data Reconciliation After Region Recovery

When a failed region recovers, its data may be stale (registrations expired,
config changes missed, CDRs from the outage period missing).

**Reconciliation steps:**

1. **Registrations:** No reconciliation needed. The recovered region's
   registration database may contain stale entries that will naturally expire.
   Users who re-registered in Region B will send a new REGISTER to Region A
   (via GeoDNS) once Region A is back in DNS. The new REGISTER overwrites the
   stale entry.

2. **Configuration:** The recovered region pulls the latest configuration from
   the database. If using PostgreSQL streaming replication, the replica catches
   up automatically when the replication link is restored. Any config changes
   made during the outage are replayed.

3. **CDR records:** CDRs generated in Region B during the outage are replicated
   to Region A's database once replication resumes. CDRs for calls that were
   active in Region A at the time of failure may be incomplete (no end time,
   no final duration). A cleanup job should mark these as `status = 'interrupted'`
   with the last known timestamp.

4. **Recordings:** Recordings for calls in the failed region are on the failed
   region's disk (or S3 if async upload was configured). If using local storage,
   recordings for in-progress calls at failure time may be truncated or
   corrupted. The S3 upload path (recommended in `clustering-architecture.md`
   Section 4.4) ensures that completed recordings from before the failure are
   already safely stored.

5. **Trunk registration:** When the recovered region rejoins, the distributed
   leader election (Redis-based) determines whether trunk registration should
   move back to the recovered region. No special handling is needed; the
   standard leader election from `clustering-architecture.md` Section 5.3
   handles this automatically.

---

## 7. Latency Budget Analysis

Voice communication quality is directly affected by end-to-end latency. The ITU-T
G.114 recommendation specifies that one-way mouth-to-ear delay should not exceed
150ms for acceptable conversational quality. This section defines latency budgets
for each component in the geographic redundancy architecture.

### 7.1 SIP Signaling Latency

SIP signaling (INVITE, 200 OK, ACK, BYE) is less latency-sensitive than media
because it occurs at call setup and teardown, not during the conversation.

| Metric | Target | Rationale |
|--------|--------|-----------|
| Intra-region SIP RTT | < 20ms | LAN or same-datacenter network |
| Cross-region SIP RTT | < 200ms | Acceptable for call setup; not perceived by users |
| INVITE-to-180 Ringing | < 3s | User expectation for call progress indication |
| INVITE-to-200 OK | < 10s | Includes callee ring time; not a network metric |

Cross-region SIP signaling latency of 200ms adds less than 0.2 seconds to call
setup time. This is imperceptible to users who are already expecting a multi-
second delay while the phone rings.

### 7.2 RTP Media Latency

RTP latency is the critical constraint for geographic placement. The total
one-way latency budget from mouth to ear is 150ms.

**Latency budget breakdown (intra-region call):**

| Component | Budget | Typical |
|-----------|--------|---------|
| Codec encoding/decoding | 20-40ms | 20ms (Opus), 0.125ms (G.711) |
| Jitter buffer | 20-60ms | 40ms adaptive |
| Network (phone to RustPBX) | 5-30ms | 10ms |
| RustPBX processing (MediaBridge) | < 2ms | 1ms |
| Network (RustPBX to phone) | 5-30ms | 10ms |
| **Total** | **50-160ms** | **~80ms** |

**Latency budget breakdown (cross-region call, Option 1 media relay):**

| Component | Budget | Typical |
|-----------|--------|---------|
| Codec encoding/decoding | 20-40ms | 20ms |
| Jitter buffer | 20-60ms | 40ms |
| Network (Phone A to RustPBX-A) | 5-30ms | 10ms |
| RustPBX-A processing | < 2ms | 1ms |
| Network (RustPBX-A to RustPBX-B) | 30-80ms | 40ms (US coast-to-coast) |
| RustPBX-B processing | < 2ms | 1ms |
| Network (RustPBX-B to Phone B) | 5-30ms | 10ms |
| **Total** | **80-240ms** | **~120ms** |

At 120ms typical, cross-region calls are within the G.114 150ms threshold but
leave little margin. For international cross-region links (e.g., US to Europe,
70-100ms one-way network latency), the total may exceed 150ms. This reinforces
the recommendation that media should stay within a region whenever possible.

### 7.3 Registration Refresh and Failover Detection

SIP REGISTER refresh interval determines how quickly a region failure is
detected from the user agent's perspective.

| Parameter | Recommended Value | Rationale |
|-----------|------------------|-----------|
| Registration expiry | 120-300s | Standard SIP range; 120s for faster failover |
| Registration refresh | 50-80% of expiry | Phone re-registers before expiry |
| Failover detection | expiry + retry timeout | 120s expiry + 32s retry = ~152s worst case |

**Recommended:** Set `registration_expires = 120` in the RustPBX config.
This means phones re-register every 60-96 seconds (50-80% of 120s). If a region
fails, the phone's next registration attempt fails and triggers DNS re-
resolution within at most 120 seconds.

For faster failover detection, the phone can be configured with shorter expiry
(60s), but this increases registration traffic proportionally.

### 7.4 Database Replication Lag

| Data Type | Maximum Acceptable Lag | Rationale |
|-----------|----------------------|-----------|
| Configuration | < 5s | Admin changes should propagate quickly but are not call-path critical |
| CDR / call records | < 60s | Reporting delay is acceptable; data integrity matters more than speed |
| Presence | < 2s | User status should be reasonably current for BLF indicators |
| Recordings (S3 sync) | < 15min | Not needed in real-time; post-call access is sufficient |

PostgreSQL async streaming replication typically achieves < 1s lag under normal
load, well within all targets. During high load or network congestion, lag may
increase; monitoring via `pg_stat_replication.replay_lag` is essential.

### 7.5 Latency Summary by Region Topology

| Scenario | SIP Signaling | RTP One-Way | Registration Failover | DB Replication |
|----------|--------------|-------------|----------------------|----------------|
| Same region, same node | < 5ms | < 80ms | N/A | N/A |
| Same region, different node | < 10ms | < 80ms | N/A | < 10ms |
| Cross-region (US, same continent) | < 100ms | < 120ms | 60-150s | < 5s |
| Cross-region (intercontinental) | < 200ms | < 180ms | 60-150s | < 10s |

---

## 8. Network Architecture Reference

### 8.1 Two-Region Deployment Topology

```
                           GeoDNS (Route 53 / Cloudflare)
                           pbx.example.com
                                  |
                  +---------------+---------------+
                  |                               |
           Region A (US-East)              Region B (US-West)
           +-----------------+             +-----------------+
           |  SIP LB / HAProxy |           |  SIP LB / HAProxy |
           +---------+---------+           +---------+---------+
                     |                               |
              +------+------+                 +------+------+
              |             |                 |             |
         +----+----+  +----+----+        +----+----+  +----+----+
         |RustPBX-1|  |RustPBX-2|        |RustPBX-3|  |RustPBX-4|
         +---------+  +---------+        +---------+  +---------+
              |             |                 |             |
         +----+-------------+----+       +----+-------------+----+
         |  PostgreSQL (primary) |       |  PostgreSQL (replica) |
         |  Redis (primary)      |       |  Redis (replica)      |
         +-----------------------+       +-----------------------+
                     |                               |
                     +---------- VPN / Peering ------+
                          (DB replication + inter-region SIP)
```

### 8.2 Port and Firewall Requirements

| Port | Protocol | Direction | Purpose |
|------|----------|-----------|---------|
| 5060 | UDP/TCP | Inbound | SIP signaling |
| 5061 | TCP | Inbound | SIP over TLS |
| 8080 | TCP | Inbound | HTTP admin console |
| 8443 | TCP | Inbound | HTTPS admin + WebSocket SIP |
| 20000-29999 | UDP | Inbound | RTP media (split across nodes) |
| 3478 | UDP/TCP | Inbound | STUN/TURN |
| 5432 | TCP | Inter-region | PostgreSQL replication |
| 6379 | TCP | Inter-region | Redis replication |
| 5060 | UDP/TCP | Inter-region | Inter-region SIP signaling |
| 20000-29999 | UDP | Inter-region | Inter-region RTP relay |

All inter-region traffic should traverse an encrypted VPN tunnel or private
network peering link. Do not expose database or Redis ports to the public
internet.

---

## 9. Implementation Roadmap

Geographic redundancy builds on the single-region clustering work from
`clustering-architecture.md`. The following phases are additive.

### Phase G1: Multi-Region Database Replication

**Prerequisite:** Phase 1 from `clustering-architecture.md` (PostgreSQL shared
database) is complete.

- Set up PostgreSQL streaming replication between regions.
- Configure per-region `DbLocator` instances (each region uses its own local
  PostgreSQL for registration reads/writes).
- Configure CDR replication for consolidated reporting.
- Verify SeaORM migration compatibility on replicated databases.

**Effort:** 1-2 weeks (mostly infrastructure, minimal code changes).

### Phase G2: GeoDNS and Regional Routing

- Configure GeoDNS with health-check-based failover for the SIP domain.
- Configure GeoDNS for the web/WebSocket domain.
- Set TURN server configuration per region in the WebRTC config endpoint.
- Test DNS failover by simulating a region failure.

**Effort:** 1 week (infrastructure and DNS configuration).

### Phase G3: Cross-Region Trunk Registration Coordination

- Extend the Redis-based trunk registration leader election to span regions
  (requires cross-region Redis connectivity or a shared Redis Cluster).
- Implement trunk registration failover: when the leader region fails, the
  surviving region acquires the lock and begins registering with the trunk
  provider.
- Test inbound call routing after trunk registration failover.

**Effort:** 1-2 weeks.

### Phase G4: Inter-Region Call Routing and Media Relay

- Implement inter-region SIP signaling for calls between users in different
  regions (Region A's RustPBX sends INVITE to Region B's RustPBX).
- Implement cross-region RTP relay via `MediaBridge` with remote peer.
- Test inter-region call quality and verify latency is within the budget from
  Section 7.

**Effort:** 3-4 weeks (significant media path work).

### Phase G5: Operational Hardening

- Implement region-level health monitoring dashboard.
- Add cross-region failover alerting (PagerDuty, Slack, etc.).
- Create runbooks for region failure, recovery, and data reconciliation.
- Perform a full failover drill: take down one region, verify service continuity,
  bring the region back, verify data reconciliation.
- Document the latency characteristics observed in production.

**Effort:** 1-2 weeks.

---

## 10. Risks and Open Questions

1. **Cross-region Redis:** The clustering architecture assumes a single Redis
   instance. For geographic redundancy, Redis must either span regions (Redis
   Cluster with cross-region members, adding latency to every Redis operation)
   or be per-region with a separate coordination mechanism for cross-region
   state (trunk registration leader election). Per-region Redis with cross-region
   pub/sub bridging is the likely solution.

2. **Trunk provider limitations:** Not all SIP trunk providers support
   multi-region registration or geographic call routing. The design must
   accommodate single-trunk-single-region as the baseline and multi-region as
   an optimization.

3. **CDR consistency during split-brain:** If both regions are operational but
   the inter-region link is down, both regions generate CDRs independently.
   When the link recovers, CDR replication must handle any sequence gaps or
   timestamp ordering issues. Using globally unique session IDs (UUIDs)
   prevents primary key conflicts.

4. **WebRTC ICE and region affinity:** WebRTC clients perform ICE negotiation
   which selects the best media path. If TURN servers are regional but the
   signaling server is in a different region (e.g., during partial failover),
   the ICE candidates may prefer a TURN server in the wrong region. The WebRTC
   config endpoint must always return TURN URIs for the region serving the
   signaling connection.

5. **Cost:** Multi-region deployment roughly doubles infrastructure costs
   (servers, bandwidth, database instances). The active-active model amortizes
   this by utilizing both regions for production traffic, but the baseline cost
   is still significantly higher than single-region.

6. **Inter-region bandwidth for media:** Cross-region RTP relay for inter-region
   calls consumes bandwidth on the inter-region link. At 80 kbps per G.711
   call (both directions), 100 concurrent cross-region calls require ~16 Mbps.
   This is modest but should be provisioned explicitly on the inter-region link.
