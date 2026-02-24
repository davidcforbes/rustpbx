# RustPBX Clustering Architecture Design

## Document Metadata

| Field | Value |
|-------|-------|
| Task | rpbx-mwi.1, rpbx-mwi.2 |
| Status | Draft |
| Created | 2026-02-23 |

---

## 1. Current Architecture Summary

Before designing the clustering solution, it is important to understand what exists today. RustPBX is a single-process SIP proxy and B2BUA written in Rust, built on `rsipstack` for SIP transaction and dialog management and `rustrtc` for RTP/WebRTC media handling.

### 1.1 SIP Proxy Layer (`src/proxy/`)

The SIP server (`SipServer` / `SipServerInner` in `src/proxy/server.rs`) is the central orchestration point. It owns:

- **Endpoint**: Binds UDP, TCP, TLS, and WebSocket SIP transports on configurable ports. A single `Endpoint` handles all inbound SIP transactions.
- **ProxyModule pipeline**: A chain of modules (`acl`, `auth`, `registrar`, `call`, `trunk_register`, `presence`) that process each SIP transaction sequentially via `on_transaction_begin` / `on_transaction_end`.
- **Locator** (`src/proxy/locator.rs`): Stores SIP registrations (AoR-to-contact bindings). Two implementations exist:
  - `MemoryLocator` -- in-process `HashMap<String, HashMap<String, Location>>` protected by a `tokio::sync::Mutex`. This is the default.
  - `DbLocator` (`src/proxy/locator_db.rs`) -- SeaORM-backed table `rustpbx_locations` with columns for AoR, username, realm, destination, transport, expires, and timestamps. Supports MySQL/PostgreSQL/SQLite.
- **UserBackend** (`src/proxy/user*.rs`): Authenticates SIP users. Backends include memory, HTTP, plain-file, database, and extension.
- **ProxyDataContext** (`src/proxy/data.rs`): A runtime snapshot holder for trunks, routes, queues, and ACL rules. Loaded from config files, TOML includes, and/or a database. Uses `RwLock` for concurrent read access.
- **ActiveProxyCallRegistry** (`src/proxy/active_call_registry.rs`): In-process `Mutex<HashMap>` tracking active call sessions and their dialog handles.

### 1.2 Call Session Layer (`src/proxy/proxy_call/`)

Each call is managed by a `CallSession` (`session.rs`) that holds:

- `CallContext` (immutable): session ID, dialplan, media config, caller/callee identities, cookie.
- `CallSessionShared` (mutable): phase, ring/answer times, error state, current target.
- `CallSessionHandle`: An `mpsc::UnboundedSender<SessionAction>` channel for controlling the call from external code (accept, transfer, hangup, re-INVITE, etc.).
- `MediaBridge` (`media_bridge.rs`): Connects two `MediaPeer` legs, forwarding RTP packets between them, with optional recording and call quality monitoring.

### 1.3 Media Layer (`src/media/`)

- `RtcTrack` / `RtpTrackBuilder`: Creates `rustrtc::PeerConnection` instances for each call leg. Each PeerConnection binds a local UDP port from a configured range (`rtp_start_port`..`rtp_end_port`).
- `MediaBridge`: Receives RTP from leg A's `PeerConnection`, optionally transcodes, records, and forwards to leg B.
- Media is always relayed through the RustPBX process (B2BUA model with `MediaProxyMode::All/Auto/Nat`).

### 1.4 Database (`src/models/`)

- Default: `sqlite://rustpbx.sqlite3` (single-file, single-writer).
- Supports MySQL via `mysql://` URLs with auto-creation of the database.
- SeaORM with `sea_orm_migration` for schema management.
- Tables: `call_records`, `extensions`, `departments`, `sip_trunks`, `routing`, `policies`, `frequency_limits`, `users`, `presence`, `voicemail`, `voicemail_greetings`.
- The `database_url` config key drives the ORM connection for the main application database.
- The `DbLocator` uses its own separate `database_url` configured under `[proxy.locator]`.

### 1.5 State That Is Currently Node-Local

| State | Storage | Scope |
|-------|---------|-------|
| SIP registrations | `MemoryLocator` (default) or `DbLocator` | Per-node or shared DB |
| Active call sessions | `ActiveProxyCallRegistry` (in-memory `HashMap`) | Per-node only |
| Dialog state (SDP, tags) | `rsipstack::DialogLayer` (in-memory) | Per-node only |
| Media bridges (RTP) | `MediaBridge` with `PeerConnection` UDP sockets | Per-node only |
| Trunk/route/queue config | `ProxyDataContext` (in-memory, loaded from files/DB) | Per-node, reloadable |
| Presence | `PresenceManager` (in-memory, backed by DB) | Per-node + DB |
| Call records | Written to DB / S3 / HTTP on call completion | Shared (DB) |
| Call quality metrics | `CallQuality` (in-memory per-bridge) | Per-node only |
| SIP flow traces | Local files or remote UDP/HTTP | Per-node or central |

---

## 2. SIP Load Balancing Architecture

### 2.1 Overview

SIP load balancing is fundamentally different from HTTP load balancing due to SIP's stateful dialog model. A SIP dialog (INVITE -> 200 OK -> ACK -> ... -> BYE) must remain pinned to the same proxy node for the duration of the call. REGISTER bindings are also stateful: subsequent requests for a registered user must route to the node that holds (or can look up) the binding.

The recommended architecture is a two-tier model:

```
                         +-----------+
          DNS SRV / A    |  Clients  |
          records        +-----------+
               |               |
               v               v
      +--------+--------+--------+--------+
      |     L4 Load Balancer (HAProxy)    |
      |   (SIP-aware, dialog-sticky)      |
      +--------+--------+--------+--------+
               |               |
         +-----+-----+  +-----+-----+
         | RustPBX-1  |  | RustPBX-2  |
         | (active)   |  | (active)   |
         +-----+------+  +-----+------+
               |               |
         +-----+---------------+------+
         |    Shared State Layer      |
         |  (PostgreSQL + Redis)      |
         +----------------------------+
```

### 2.2 DNS-Based Load Balancing

DNS SRV records (RFC 3263) provide the first layer of distribution and are native to SIP. SIP user agents resolve a domain like `sip:user@pbx.example.com` into one or more transport/host/port targets using:

1. NAPTR records (optional): select transport (UDP, TCP, TLS)
2. SRV records: `_sip._udp.pbx.example.com` with priority and weight
3. A/AAAA records: final IP resolution

**Configuration example:**

```
; Equal-weight distribution across two nodes
_sip._udp.pbx.example.com. 300 IN SRV 10 50 5060 node1.pbx.example.com.
_sip._udp.pbx.example.com. 300 IN SRV 10 50 5060 node2.pbx.example.com.

; TCP transport
_sip._tcp.pbx.example.com. 300 IN SRV 10 50 5060 node1.pbx.example.com.
_sip._tcp.pbx.example.com. 300 IN SRV 10 50 5060 node2.pbx.example.com.

node1.pbx.example.com. 300 IN A 10.0.0.10
node2.pbx.example.com. 300 IN A 10.0.0.11
```

**Advantages:**
- No additional infrastructure component.
- SIP UAs that properly implement RFC 3263 will fail over to the backup target on timeout.
- Can set weights for gradual traffic shifting (canary deployments).

**Limitations:**
- No health checking. DNS TTL governs failover speed (trade-off between TTL and DNS query load).
- Many SIP phones cache DNS results aggressively and do not re-resolve on failure.
- Does not solve dialog affinity: a re-INVITE or BYE could land on the wrong node.

**Recommendation:** Use DNS SRV as the baseline for initial contact distribution and as a failover mechanism, but combine with an L4 load balancer for production reliability.

### 2.3 L4 Load Balancer (HAProxy / nginx)

An L4 (transport-layer) load balancer sits in front of the RustPBX nodes and distributes SIP traffic.

#### HAProxy (Recommended)

HAProxy supports both UDP and TCP frontends. For SIP:

```
frontend sip_udp
    bind *:5060 udp
    mode udp
    default_backend rustpbx_udp

frontend sip_tcp
    bind *:5060
    mode tcp
    default_backend rustpbx_tcp

frontend sip_wss
    bind *:443 ssl crt /etc/haproxy/certs/pbx.pem
    mode tcp
    default_backend rustpbx_ws

backend rustpbx_udp
    mode udp
    balance source          # IP-based stickiness
    hash-type consistent
    server node1 10.0.0.10:5060 check
    server node2 10.0.0.11:5060 check

backend rustpbx_tcp
    mode tcp
    balance source
    hash-type consistent
    stick-table type ip size 100k expire 30m
    stick on src
    server node1 10.0.0.10:5060 check
    server node2 10.0.0.11:5060 check

backend rustpbx_ws
    mode tcp
    balance source
    server node1 10.0.0.10:8080 check
    server node2 10.0.0.11:8080 check
```

#### Kamailio as SIP-Aware Load Balancer (Alternative)

For full SIP awareness (dialog tracking, Via-based routing, topology hiding):

```
# Kamailio as a stateless SIP proxy in front of RustPBX cluster
modparam("dispatcher", "list_file", "/etc/kamailio/dispatcher.list")
modparam("dispatcher", "ds_ping_method", "OPTIONS")
modparam("dispatcher", "ds_ping_interval", 10)
modparam("dispatcher", "ds_probing_mode", 1)

# dispatcher.list:
# 1 sip:10.0.0.10:5060 0
# 1 sip:10.0.0.11:5060 0
```

Kamailio provides:
- Call-ID based hashing (ensures all messages in a dialog go to the same node).
- Active health checking via SIP OPTIONS pings.
- Automatic failover when a node stops responding.

**Recommendation:** Start with HAProxy source-IP hashing for simplicity. Move to Kamailio if full SIP-level dialog affinity is needed before implementing shared dialog state in Phase 3.

### 2.4 Dialog Affinity and Mid-Dialog Requests

SIP dialogs require that all subsequent requests (re-INVITE, UPDATE, BYE, INFO, REFER) reach the same proxy that established the dialog. RustPBX achieves this today via Record-Route headers -- the proxy inserts itself into the SIP routing path so that in-dialog requests traverse back through it.

In a multi-node setup, the challenge is that a client's re-INVITE may arrive at the load balancer and be sent to a different node than the one that processed the original INVITE.

**Strategies (in order of implementation):**

1. **Source-IP stickiness** (HAProxy `balance source`): Works when clients consistently send from the same IP. Sufficient for most phone/softphone deployments. Breaks if clients are behind symmetric NAT pools.

2. **Call-ID hashing** (Kamailio dispatcher): Hash on the `Call-ID` header. All messages in the same dialog go to the same backend. This is the most reliable L4-level approach.

3. **Shared dialog state** (Phase 3): Any node can handle any in-dialog request by looking up the dialog in a shared store (Redis). The receiving node either processes the request directly or forwards it internally to the owning node.

### 2.5 Registration Stickiness

REGISTER requests create bindings that must be queryable by any node routing calls to that user. Two approaches:

1. **Shared registration store** (recommended): All nodes use `DbLocator` pointing at the same PostgreSQL database, or a Redis-backed locator. Any node can look up any registration.

2. **Registration broadcasting**: On receiving a REGISTER, the node stores locally and broadcasts to peers. More complex, harder to keep consistent.

### 2.6 Health Check Endpoints

RustPBX should expose HTTP health check endpoints for use by load balancers and orchestration systems.

**Proposed endpoints:**

```
GET /health          -> 200 OK (basic liveness)
GET /health/ready    -> 200 OK when fully initialized, 503 during startup/drain
GET /health/detail   -> JSON with component status
```

**Health detail response:**

```json
{
  "status": "healthy",
  "node_id": "node1",
  "uptime_seconds": 86400,
  "sip_endpoint": "ok",
  "database": "ok",
  "active_calls": 42,
  "active_registrations": 150,
  "memory_mb": 128,
  "version": "0.3.18"
}
```

The `/health/ready` endpoint is critical for graceful drain: when a node is being shut down, it returns 503 so the load balancer stops sending new traffic while existing calls complete.

### 2.7 Active-Active vs Active-Passive Topology

| Topology | Pros | Cons |
|----------|------|------|
| **Active-Active** | Full capacity utilization, horizontal scaling, no wasted resources | Requires shared state, more complex failover |
| **Active-Passive** | Simple failover, no shared state needed | Wastes standby capacity, longer failover time |

**Recommendation:** Target active-active. The shared state infrastructure (PostgreSQL + Redis) needed for active-active also enables active-passive as a simpler subset, so the engineering effort is similar. Active-active is the only topology that provides horizontal scaling for call capacity.

---

## 3. Shared State Architecture

### 3.1 State Categories and Sharing Requirements

| State | Sharing Need | Latency Tolerance | Consistency | Recommended Store |
|-------|-------------|-------------------|-------------|-------------------|
| Configuration (trunks, routes, ACLs) | Read-heavy, infrequent writes | Seconds | Eventual | PostgreSQL + file reload |
| SIP registrations | Read on every call route, write on REGISTER | < 10ms | Strong | PostgreSQL (existing `DbLocator`) or Redis |
| Active call sessions | Per-node ownership, query for admin/API | Low (intra-node) | Per-node authoritative | Redis (pub/sub for notifications) |
| Dialog state (SDP, tags, routes) | Per-node ownership, needed for failover | Very low | Per-node authoritative | Redis (optional replication) |
| Call records | Write-once on call end | Seconds | Eventual | PostgreSQL |
| Presence | Read/write | < 1s | Eventual | PostgreSQL + Redis cache |
| Media (RTP streams) | Per-node only (cannot share UDP sockets) | N/A | N/A | Local only |
| Recordings | Write during call, read after | Seconds | Eventual | Shared filesystem / S3 |

### 3.2 State Store Selection

#### PostgreSQL (Primary persistent store)

PostgreSQL replaces SQLite as the shared database for all persistent state. RustPBX already supports MySQL via SeaORM; adding PostgreSQL is a configuration change (the ORM abstracts the dialect).

**Used for:**
- Application database (call records, extensions, departments, trunks, routes, policies, users, voicemail)
- Registration store (existing `DbLocator` with `rustpbx_locations` table)
- Presence state

**Connection pooling:** Use `deadpool-postgres` or rely on SeaORM's built-in connection pool with `max_connections` tuned per node (e.g., 10-20 connections per node).

**Schema consideration:** The current `DbLocator` creates its own database connection from `[proxy.locator]` config. For clustering, unify this to use the main `database_url` pointing at PostgreSQL, so registrations and call records share the same database.

#### Redis (Fast ephemeral state and pub/sub)

Redis provides sub-millisecond access for ephemeral state that is too hot for PostgreSQL query paths.

**Used for:**
- Registration cache (read-through cache in front of PostgreSQL)
- Active call session index (which node owns which call)
- Dialog-to-node mapping (for mid-dialog request routing)
- Pub/sub channel for inter-node events (registration changes, config reload signals, presence updates)
- Distributed locks (for trunk registration deduplication)

**High availability:** Redis Sentinel or Redis Cluster for failover. A single Redis Sentinel setup with one primary and one replica is sufficient for small clusters (2-4 nodes).

**Data structures:**

```
# Registration cache (hash per user)
HSET reg:alice@pbx.example.com contact1 '{"aor":"sip:alice@...","dest":"10.0.0.10:5060","expires":3600,"ts":1234567890}'
EXPIRE reg:alice@pbx.example.com 3600

# Active call index (hash: session_id -> node_id)
HSET active_calls <session-id> '{"node":"node1","caller":"alice","callee":"bob","started":1234567890}'

# Dialog-to-node mapping
SET dialog:<call-id>:<local-tag>:<remote-tag> node1 EX 7200

# Node registry
HSET cluster:nodes node1 '{"addr":"10.0.0.10","status":"active","last_heartbeat":1234567890}'
EXPIRE cluster:nodes:node1 30

# Pub/sub channels
PUBLISH cluster:registrations '{"event":"register","user":"alice","node":"node1"}'
PUBLISH cluster:config_reload '{"timestamp":1234567890}'
```

#### etcd (Alternative for service discovery)

etcd could be used for service discovery and leader election. However, Redis Sentinel already provides the leader election capability needed for trunk registration deduplication, and PostgreSQL provides the persistent configuration store. Adding etcd would introduce another operational dependency without clear benefit at the 2-8 node scale.

**Recommendation:** Do not adopt etcd unless the deployment grows beyond 8 nodes or Kubernetes-native service discovery is needed.

### 3.3 Registration Sync Between Nodes

**Current state:** `MemoryLocator` is purely in-process. `DbLocator` stores registrations in a database but has no cache invalidation between nodes.

**Target architecture:**

```
                 REGISTER from client
                        |
                        v
               +--------+--------+
               |    Node 1       |
               | RegistrarModule |
               +--------+--------+
                        |
          +-------------+-------------+
          |                           |
          v                           v
  +-------+-------+         +--------+--------+
  |  PostgreSQL    |         |     Redis       |
  | (persistent)   |         | (cache + notify)|
  +-------+-------+         +--------+--------+
          |                           |
          |                    PUBLISH cluster:registrations
          |                           |
          |                           v
          |                  +--------+--------+
          +----------------->|    Node 2       |
                             | (cache update)  |
                             +-----------------+
```

**Flow:**

1. Client sends REGISTER to Node 1.
2. `RegistrarModule` authenticates and calls `locator.register()`.
3. The locator writes to PostgreSQL (authoritative store) and Redis (cache).
4. The locator publishes a `cluster:registrations` event on Redis pub/sub.
5. Node 2 receives the pub/sub event and invalidates/updates its local cache.
6. When Node 2 needs to route a call to the registered user, it checks Redis first, falls back to PostgreSQL.

**Implementation:** Create a new `ClusteredLocator` that wraps `DbLocator` and adds:
- A Redis connection for caching and pub/sub.
- A local in-memory LRU cache (bounded, e.g., 10,000 entries) for hot-path lookups.
- A background task subscribed to `cluster:registrations` for cache invalidation.

### 3.4 Dialog/Session Replication Strategy

Full dialog replication (replicating SDP offers/answers, Route sets, CSeq counters) to enable seamless mid-dialog failover is complex and expensive. The recommended approach is **dialog ownership with routing hints** rather than full replication.

**Model:**

- Each active call is "owned" by the node that processed the initial INVITE.
- The owning node writes a dialog-to-node mapping into Redis: `dialog:<call-id> -> node-id`.
- If a mid-dialog request (re-INVITE, BYE) arrives at the wrong node, that node looks up the mapping in Redis and either:
  - (a) Proxies the request internally to the correct node (inter-node SIP forwarding), or
  - (b) Returns a 302 Moved Temporarily to redirect the request (simpler but less transparent).

**Failover behavior:** If the owning node dies, the dialog mapping becomes stale. The surviving node detects the dead peer (via heartbeat timeout), marks its calls as orphaned, and relies on SIP timers (dialog timeout, session timer refresh failure) to naturally clean up. Calls in progress on the dead node are lost -- the phones will detect the failure via RTP timeout or session timer expiry and can re-dial.

**Full dialog replication (future consideration):** For zero-downtime failover, each node would replicate its dialog state (SDP, Route set, CSeq, media port bindings) to a peer via Redis. The surviving node would reconstruct the dialog and send a re-INVITE to redirect media to its own ports. This is engineering-intensive and should only be pursued if call continuity through node failure is a hard requirement.

### 3.5 Configuration Distribution

Configuration in RustPBX is currently loaded from TOML files and optionally from the database. The `ProxyDataContext` supports runtime reload via `reload_trunks()`, `reload_routes()`, `reload_queues()`, `reload_acl_rules()`.

**Clustering approach:**

1. **Database as source of truth:** Store trunks, routes, queues, and ACL rules in PostgreSQL. The console UI already writes to the database. Each node loads from the database on startup and on reload.

2. **Reload signaling:** When an admin changes configuration via the console, the node that processed the change publishes a `cluster:config_reload` event on Redis pub/sub. All nodes receive the event and trigger a `reload_*()` cycle.

3. **Config file override:** Nodes can still have local config file overrides (e.g., node-specific `external_ip`, `rtp_start_port` ranges). These are not shared.

4. **Node-specific config:**

```toml
# Shared (in database or shared config file)
[proxy]
modules = ["acl", "auth", "registrar", "call", "trunk_register"]

# Node-specific (local config file)
external_ip = "10.0.0.10"           # This node's public IP
rtp_start_port = 20000              # Non-overlapping RTP port range
rtp_end_port = 25000
node_id = "node1"                   # Unique node identifier

[cluster]
redis_url = "redis://10.0.0.20:6379"
node_heartbeat_interval_secs = 10
node_heartbeat_timeout_secs = 30
```

### 3.6 Database Migration: SQLite to PostgreSQL

The migration from SQLite to PostgreSQL is primarily a configuration change since SeaORM abstracts SQL dialects. Key considerations:

1. **Schema compatibility:** SeaORM migrations are dialect-aware. The existing migrations in `src/models/migration.rs` should work for PostgreSQL. Test each migration against PostgreSQL in CI.

2. **Data migration:** For existing SQLite deployments, provide a one-time migration script:
   ```bash
   rustpbx --migrate-db --source sqlite://rustpbx.sqlite3 --target postgresql://user:pass@host/rustpbx
   ```

3. **Connection string format:**
   ```toml
   database_url = "postgresql://rustpbx:password@db.example.com:5432/rustpbx"
   ```

4. **SQLite limitations removed:** No more single-writer bottleneck. No file locking issues. Concurrent writes from multiple nodes are handled by PostgreSQL's MVCC.

---

## 4. Media Path Considerations

### 4.1 RTP Relay Is Per-Node

RTP media is processed in-kernel (UDP sockets) and in-process (`PeerConnection` in `rustrtc`). Each `MediaBridge` binds local UDP ports for leg A and leg B. These sockets are bound to the node's IP address and cannot be shared across nodes.

**Consequence:** A call's media always flows through the node that established the call. If that node fails, media stops immediately. There is no practical way to "migrate" a live RTP stream to another node without a re-INVITE.

### 4.2 Re-INVITE on Failover

If dialog state is replicated (Phase 4), a surviving node could attempt call recovery:

1. Detect peer node failure.
2. Look up replicated dialog state from Redis.
3. Allocate new local RTP ports.
4. Send re-INVITE to both call legs with updated SDP (new IP/port for media).
5. Both phones update their RTP targets, media resumes through the surviving node.

**Latency:** This process takes 1-5 seconds (re-INVITE round-trip to both sides). During this window, there is no audio. Most VoIP users would perceive a brief silence followed by audio resumption -- this is acceptable for a failover scenario.

**Prerequisites:**
- Full dialog state replication (SDP, Route set, CSeq, authentication state).
- Non-overlapping RTP port ranges across nodes (already configurable via `rtp_start_port`/`rtp_end_port`).

### 4.3 RTP Port Range Allocation

Each node must have a non-overlapping RTP port range:

| Node | RTP Range | WebRTC Range |
|------|-----------|--------------|
| node1 | 20000-24999 | 30000-34999 |
| node2 | 25000-29999 | 35000-39999 |
| node3 | 40000-44999 | 45000-49999 |

This is already supported by the existing `rtp_start_port` / `rtp_end_port` and `webrtc_port_start` / `webrtc_port_end` config keys. No code changes needed.

### 4.4 Recording Aggregation

Recordings are currently written to the local filesystem (`config/recorders/`). In a cluster, recordings from different nodes must be accessible centrally.

**Options:**

1. **Shared filesystem (NFS/CIFS):** Mount a shared volume at the recorder path. Simple but introduces a SPOF and potential performance issues.

2. **Object storage (S3):** RustPBX already supports S3 for call records (`CallRecordConfig::S3`). Extend recording upload to use S3. The `StorageConfig` in `src/storage/mod.rs` already supports S3 with multiple vendors (AWS, GCP, Azure, Aliyun, Minio, etc.).

3. **Post-call upload:** Record locally during the call (low latency), then upload to S3 on call completion. This is the recommended approach -- it avoids network latency during live recording while ensuring centralized storage.

**Recommendation:** Use S3-compatible storage (Minio for on-premise) for recording aggregation. Local recording during the call, background upload on completion.

### 4.5 Codec Negotiation Consistency

All nodes must offer the same set of codecs to ensure that a phone registered on node1 can call a phone registered on node2 without transcoding failures. This is enforced by sharing the `codecs` configuration across nodes (via the shared database or a common config file).

The `MediaNegotiator` in `src/media/negotiate.rs` and the codec preference lists in `RtpTrackBuilder` must use the same codec ordering on all nodes. Since codec config comes from the shared `ProxyConfig`, this is automatically consistent when all nodes load from the same database.

---

## 5. Failover Scenarios

### 5.1 Node Failure During Active Call

**Symptoms:**
- RTP audio stops (phones detect silence/packet loss).
- SIP session timer refresh fails (if session timers are enabled).
- BYE from the surviving party times out.

**Without dialog replication (Phases 1-3):**
1. Load balancer detects node failure via health check (2-10 seconds depending on interval).
2. Load balancer removes dead node from rotation.
3. Active calls on the dead node are lost. Phones detect RTP timeout (typically 30 seconds) or session timer failure.
4. Users must re-dial. Call records for in-progress calls may be lost unless a pre-write to the database occurred.

**With dialog replication (Phase 4):**
1. Surviving node detects peer failure via Redis heartbeat timeout (10-30 seconds).
2. Surviving node reads replicated dialog state from Redis.
3. Surviving node sends re-INVITE to both legs with new SDP.
4. Call resumes after a brief audio gap (1-5 seconds).

**Mitigation (Phases 1-3):** Enable SIP session timers (`session_timer = true` in config). Set `session_expires` to 90-180 seconds. This ensures that failed calls are detected and cleaned up within the timer interval rather than lingering.

### 5.2 Node Failure During Registration

**Symptoms:**
- The phone's REGISTER response is lost (timeout).
- The phone retries REGISTER, which goes to the surviving node.

**Behavior:**
- If using `DbLocator` with shared PostgreSQL, the previous registration may still exist in the database (from the dead node). The new REGISTER from the phone updates the destination to point to the phone's new contact via the surviving node.
- If using `MemoryLocator`, registrations on the dead node are lost entirely. The phone re-registers on the surviving node.

**Recommendation:** Use `DbLocator` with PostgreSQL so that registrations survive node failures. The phone simply re-registers, updating the contact destination.

### 5.3 Split-Brain Prevention

Split-brain occurs when two nodes each believe the other has failed and both claim ownership of shared resources (e.g., trunk registrations, config writes).

**Prevention strategies:**

1. **Fencing via Redis:** Use Redis `SET ... NX EX` for distributed locks. A node must acquire a lock before performing exclusive operations (e.g., trunk REGISTER to an upstream provider). If it cannot reach Redis, it must not proceed with the exclusive operation.

2. **Trunk registration leader election:** Only one node should send REGISTER to upstream trunk providers (to avoid duplicate registrations). Use a Redis-based leader election:
   ```
   SET trunk_register_leader:telnyx node1 NX EX 30
   ```
   The leader refreshes the lock every 10 seconds. If it fails, another node acquires the lock after expiry.

3. **Quorum for writes:** For configuration changes via the console, require the database write to succeed (PostgreSQL is the arbiter). If a node cannot reach PostgreSQL, it refuses configuration changes.

4. **Heartbeat protocol:**
   - Each node writes `cluster:nodes:<node-id>` to Redis every `node_heartbeat_interval_secs` (default 10s).
   - Each node monitors all peer heartbeats. If a peer's heartbeat is older than `node_heartbeat_timeout_secs` (default 30s), the peer is considered dead.
   - Dead peer's resources (trunk registration locks, active call entries) are cleaned up.

### 5.4 Graceful Node Drain for Maintenance

When taking a node offline for maintenance:

1. **Mark node as draining:**
   ```
   PUT /admin/cluster/drain
   ```
   This sets the node's health endpoint `/health/ready` to return 503.

2. **Load balancer stops sending new traffic** (within one health check interval).

3. **Existing calls continue** until natural completion or a configurable drain timeout.

4. **Registration refresh** is rejected with 503, causing phones to re-register on surviving nodes.

5. **After drain timeout or all calls complete,** the node shuts down cleanly.

**Implementation:**
- Add a `draining: AtomicBool` flag to `AppStateInner`.
- When draining, `RegistrarModule` responds to REGISTER with 503 Service Unavailable and a `Retry-After` header.
- When draining, `CallModule` rejects new INVITEs with 503 but continues to process in-dialog requests.
- The `/health/ready` endpoint checks the drain flag.

---

## 6. Implementation Phases

### Phase 1: Shared Database (PostgreSQL)

**Goal:** Replace SQLite with PostgreSQL as the shared persistent store. All nodes read/write to the same database.

**Tasks:**

1. **Test SeaORM migrations against PostgreSQL.** Run the full migration suite (`src/models/migration.rs`) against a PostgreSQL instance. Fix any dialect-specific issues (e.g., auto-increment syntax, boolean types).

2. **Update `DbLocator` to use the main `database_url`.** Currently `DbLocator` has its own connection URL. Add a configuration option to share the main database connection, avoiding duplicate connection pools.

3. **Add PostgreSQL connection string support to documentation.** Update config examples:
   ```toml
   database_url = "postgresql://rustpbx:password@db.example.com:5432/rustpbx"

   [proxy.locator]
   type = "database"
   url = "postgresql://rustpbx:password@db.example.com:5432/rustpbx"
   ```

4. **Add health check endpoints.** Implement `/health`, `/health/ready`, and `/health/detail` HTTP endpoints in the Axum router.

5. **Data migration tooling.** Create a CLI command to migrate data from an existing SQLite database to PostgreSQL.

6. **CI testing.** Add PostgreSQL to the CI matrix (Docker container) and run all integration tests against it.

**Estimated effort:** 1-2 weeks.

**Deliverables:**
- Multiple RustPBX nodes can point at the same PostgreSQL database.
- Registrations via `DbLocator` are shared across nodes.
- Call records from all nodes appear in the same database.
- Health check endpoints are available for load balancer integration.

### Phase 2: Registration Sync and Node Awareness

**Goal:** Add Redis-based caching, pub/sub for registration events, and basic cluster awareness.

**Tasks:**

1. **Add Redis dependency.** Integrate `redis` crate (async, with connection pooling via `deadpool-redis` or `bb8-redis`).

2. **Implement `ClusteredLocator`.** New locator that wraps `DbLocator` with:
   - Redis write-through cache for registrations.
   - Redis pub/sub subscription for `cluster:registrations` events.
   - Local in-memory LRU cache (e.g., `moka` crate) with TTL-based expiry.
   - Lookup path: local cache -> Redis -> PostgreSQL.

3. **Node heartbeat.** Each node writes a heartbeat to Redis every N seconds. Implement a background task that monitors peer heartbeats and logs warnings on timeout.

4. **Configuration reload signaling.** When trunks/routes/queues are modified via the console API, publish a `cluster:config_reload` event. All nodes subscribe and trigger `ProxyDataContext::reload_*()`.

5. **Trunk registration leader election.** Implement Redis-based distributed lock for `TrunkRegistrationModule`. Only the lock holder sends REGISTER to upstream providers.

6. **Cluster configuration section.** Add `[cluster]` config:
   ```toml
   [cluster]
   enabled = true
   node_id = "node1"               # Unique per node
   redis_url = "redis://10.0.0.20:6379"
   heartbeat_interval_secs = 10
   heartbeat_timeout_secs = 30
   registration_cache_size = 10000
   registration_cache_ttl_secs = 300
   ```

**Estimated effort:** 2-3 weeks.

**Deliverables:**
- Sub-millisecond registration lookups via Redis cache.
- Automatic cache invalidation when registrations change on any node.
- Only one node registers with upstream trunk providers.
- Configuration changes propagate to all nodes within seconds.

### Phase 3: Dialog Awareness and Graceful Operations

**Goal:** Enable mid-dialog request routing across nodes and graceful maintenance procedures.

**Tasks:**

1. **Dialog-to-node mapping.** When a call is established, write `dialog:<call-id> -> node-id` to Redis with a TTL matching the expected call duration (e.g., 2 hours, refreshed periodically).

2. **Mid-dialog request routing.** When a node receives a mid-dialog request (re-INVITE, BYE, INFO) for a dialog it does not own:
   - Look up the owning node in Redis.
   - Forward the SIP request to the owning node via an internal SIP connection (or HTTP API call).
   - Return the response to the original sender.

3. **Active call index.** Write active call metadata to Redis (`active_calls` hash). This enables cluster-wide call listing via the admin API.

4. **Graceful drain.** Implement the drain procedure described in section 5.4:
   - `draining` flag in `AppStateInner`.
   - `/admin/cluster/drain` API endpoint.
   - 503 responses for new REGISTER and INVITE during drain.
   - Configurable drain timeout.

5. **Cluster admin API.** Add endpoints:
   - `GET /admin/cluster/nodes` -- list all nodes with status.
   - `GET /admin/cluster/calls` -- list active calls across all nodes.
   - `POST /admin/cluster/drain` -- start drain on this node.
   - `POST /admin/cluster/undrain` -- cancel drain.

**Estimated effort:** 3-4 weeks.

**Deliverables:**
- Mid-dialog requests (BYE, re-INVITE) are correctly routed even when landing on the wrong node.
- Graceful maintenance with zero dropped calls for existing calls.
- Cluster-wide admin visibility.

### Phase 4: Full Active-Active with Optional Call Recovery

**Goal:** Production-grade active-active clustering with optional call continuity through node failure.

**Tasks:**

1. **Dialog state replication (optional).** For each active call, periodically replicate dialog state to Redis:
   - Dialog ID (Call-ID, local/remote tags).
   - SDP (local and remote).
   - Route set (Record-Route headers).
   - CSeq counters.
   - Media port bindings.
   - Codec negotiation results.

   Use a Redis hash per dialog with a 2-hour TTL.

2. **Call recovery on failover (optional).** When a peer node is detected as dead:
   - Read its replicated dialog state from Redis.
   - For each active dialog, allocate new local RTP ports.
   - Send re-INVITE to both legs with updated SDP.
   - Establish new `MediaBridge` with the new ports.

3. **Split-brain hardening.** Implement proper fencing:
   - Nodes that cannot reach Redis enter read-only mode (process existing calls but reject new ones).
   - Trunk registration lock uses Redis with fencing tokens.

4. **Load-based routing.** The load balancer (or Kamailio dispatcher) can use the active call count from the health detail endpoint to distribute new calls to the least-loaded node:
   ```
   GET /health/detail -> {"active_calls": 42, ...}
   ```

5. **Capacity planning metrics.** Expose Prometheus-compatible metrics:
   - `rustpbx_active_calls` (gauge)
   - `rustpbx_registrations` (gauge)
   - `rustpbx_calls_total` (counter)
   - `rustpbx_call_failures_total` (counter)
   - `rustpbx_node_status` (gauge: 0=down, 1=active, 2=draining)

**Estimated effort:** 4-6 weeks.

**Deliverables:**
- Full active-active clustering with horizontal scaling.
- Optional call continuity through node failure (1-5 second audio gap).
- Production monitoring and alerting via Prometheus metrics.

---

## 7. New and Modified Source Files

This section maps the design to specific code changes.

### New Files

| File | Purpose |
|------|---------|
| `src/proxy/locator_clustered.rs` | `ClusteredLocator` wrapping DbLocator + Redis cache + pub/sub |
| `src/cluster/mod.rs` | Cluster module: node heartbeat, peer detection, leader election |
| `src/cluster/redis.rs` | Redis connection management, pub/sub subscription |
| `src/cluster/dialog_router.rs` | Dialog-to-node mapping, inter-node SIP forwarding |
| `src/cluster/drain.rs` | Graceful drain state machine |
| `src/handler/health.rs` | Health check HTTP endpoints |
| `src/config.rs` (modified) | Add `ClusterConfig` struct |

### Modified Files

| File | Change |
|------|--------|
| `src/config.rs` | Add `ClusterConfig` with `node_id`, `redis_url`, heartbeat settings |
| `src/proxy/server.rs` | Accept `ClusteredLocator` option in builder; pass cluster context |
| `src/proxy/registrar.rs` | Publish registration events via cluster pub/sub |
| `src/proxy/trunk_register.rs` | Acquire distributed lock before registering with upstream |
| `src/proxy/active_call_registry.rs` | Optionally write to Redis for cross-node visibility |
| `src/proxy/proxy_call/session.rs` | Write dialog-to-node mapping on call establishment |
| `src/proxy/data.rs` | Subscribe to config reload events from peer nodes |
| `src/app.rs` | Initialize cluster module, connect Redis, start heartbeat |
| `src/models/mod.rs` | Ensure PostgreSQL compatibility for all migrations |

---

## 8. Risks and Open Questions

1. **Redis as a dependency:** Adding Redis increases operational complexity. For small deployments (2 nodes), Redis may be overkill. Consider making Redis optional: Phase 1 works without Redis (just shared PostgreSQL), and Phases 2-4 require it.

2. **PostgreSQL performance for registrations:** With thousands of phones re-registering every 60 seconds, the `rustpbx_locations` table will see high write throughput. Index on `(username, realm)` is already present. Monitor for lock contention and consider `INSERT ... ON CONFLICT UPDATE` (upsert) instead of the current check-then-insert/update pattern in `DbLocator`.

3. **Inter-node SIP forwarding:** Phase 3 requires one RustPBX node to forward SIP messages to another. This could be done via:
   - Direct SIP (node1 sends the message to node2's SIP port) -- simpler but requires SIP routing awareness.
   - HTTP API (node1 calls node2's REST API to trigger an action) -- more decoupled but adds latency.

   The SIP approach is more natural for mid-dialog requests. The HTTP approach is better for admin operations.

4. **WebSocket connections:** WebSocket SIP clients maintain persistent connections. These connections are inherently sticky to the node they connected to. If that node fails, the WebSocket closes and the client must reconnect (landing on the surviving node). WebRTC re-establishment is required.

5. **Recording during failover:** If a call is being recorded and the node fails, the partial recording file on the dead node's disk may be lost. Using S3 upload with periodic flushing (every N seconds) reduces the data loss window.

6. **Trunk registration duplication:** If the leader election for trunk registration fails (Redis unavailable), two nodes might both register with the upstream provider. Most SIP registrars handle this gracefully (last registration wins), but it could cause brief routing instability.
