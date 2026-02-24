# Configuration Sync Across Cluster

## Document Metadata

| Field | Value |
|-------|-------|
| Task | rpbx-mwi.7 |
| Status | Draft |
| Created | 2026-02-24 |
| Parent | [clustering-architecture.md](clustering-architecture.md) (rpbx-mwi.1, rpbx-mwi.2) |

---

## 1. Problem Statement

RustPBX manages several categories of runtime configuration that govern call routing,
trunk selection, access control, user authentication, and recording behavior. Today
these settings are loaded from TOML files and/or a database into in-process data
structures at startup, with manual reload available via API.

In a multi-node cluster, an administrator changes a route through the console UI (or
REST API). That HTTP request lands on one node, which writes the change to the
database and updates its own `ProxyDataContext`. The other nodes continue operating
with stale configuration until they are individually reloaded. This creates a window
where nodes disagree on routing policy, trunk availability, ACL rules, and recording
behavior.

**Requirements:**

1. Configuration changes made on any node must propagate to all cluster nodes within
   a bounded time window (target: under 10 seconds, best-effort: under 2 seconds).
2. Propagation must not disrupt active calls.
3. The mechanism must work with the existing `ProxyDataContext` architecture, which
   uses `RwLock<T>` for each config section.
4. Node-specific settings (e.g., `external_ip`, `rtp_start_port`, `node_id`) must
   remain local and never be overwritten by cluster sync.
5. The solution must degrade gracefully: if the notification channel fails, nodes
   must still converge via periodic polling.

**Config sections affected:**

| Section | DB Table | Runtime Structure | Write Path |
|---------|----------|-------------------|------------|
| Routes | `rustpbx_routes` | `RwLock<Vec<RouteRule>>` | Console UI, REST API |
| Trunks | `rustpbx_sip_trunks` | `RwLock<HashMap<String, TrunkConfig>>` | Console UI, REST API |
| Extensions/Users | `rustpbx_extensions` | `ExtensionUserBackend` (LRU cache) | Console UI, REST API |
| ACL rules | config files (no DB table yet) | `RwLock<Vec<String>>` | Config files, REST API |
| Recording policies | Embedded in trunk/route `metadata` JSON | Per-trunk `RecordingPolicy` | Console UI |
| Queues | config files + generated TOML | `RwLock<HashMap<String, RouteQueueConfig>>` | Config files, REST API |

---

## 2. Approach Comparison

Five approaches were evaluated. The comparison assumes a 2-8 node RustPBX cluster
with a shared PostgreSQL database already in place (per the clustering architecture
design in rpbx-mwi.1/mwi.2).

### 2.1 Centralized Database Config (Recommended)

All configuration lives in PostgreSQL. Each node polls or is notified when data
changes, then reloads the affected `ProxyDataContext` section.

| Attribute | Assessment |
|-----------|------------|
| Consistency | Strong -- single source of truth, no divergence |
| Latency | Seconds (polling) or sub-second (with notification layer) |
| Complexity | Low -- RustPBX already reads trunks and routes from DB |
| Infrastructure | PostgreSQL (already required for clustering) |
| Failure mode | If DB is down, nodes keep last-known config; no new changes |

**Pros:**
- No new infrastructure beyond the shared PostgreSQL already planned.
- `ProxyDataContext` already has `load_trunks_from_db()` and `load_routes_from_db()`.
- The console UI already writes to the database; the only missing piece is notifying
  peer nodes to reload.
- Atomic transactions ensure config is always in a consistent state.
- Naturally supports optimistic locking via `updated_at` timestamps.

**Cons:**
- Polling adds latency (configurable, typically 5-10 seconds).
- PostgreSQL becomes a single point of failure for config writes (mitigated by
  PostgreSQL replication in production).
- ACL rules are not yet in the database -- requires a migration to add a
  `rustpbx_acl_rules` table or similar.

### 2.2 Config Push via Message Queue (NATS / Redis Pub/Sub)

When a config change occurs, the originating node publishes the full config delta (or
a reload signal) to a message queue. All subscribed nodes receive it and apply the
change.

| Attribute | Assessment |
|-----------|------------|
| Consistency | Eventual -- missed messages cause drift until next full sync |
| Latency | Sub-second (pub/sub is near-instant) |
| Complexity | Medium -- requires message broker integration, serialization |
| Infrastructure | Redis (already planned) or NATS (new dependency) |
| Failure mode | If broker is down, changes are lost; requires fallback polling |

**Pros:**
- Very low latency for propagation.
- Redis is already planned for the clustering architecture (registration cache, heartbeat).
- Fan-out is automatic -- one publish reaches all subscribers.

**Cons:**
- Redis pub/sub is fire-and-forget: if a node is temporarily disconnected, it misses
  the message and has no way to recover without a separate mechanism.
- Requires a fallback (polling or version counter) to handle missed messages.
- Adds operational dependency on Redis availability for config correctness.
- Two sources of truth (DB for persistence, pub/sub for propagation) can diverge.

### 2.3 etcd / Consul Key-Value Store

Store configuration in a distributed KV store with watch capabilities. Nodes subscribe
to key prefixes and receive change notifications.

| Attribute | Assessment |
|-----------|------------|
| Consistency | Strong (etcd uses Raft consensus) |
| Latency | Sub-second (watch streams) |
| Complexity | High -- new dependency, data model mapping, operational overhead |
| Infrastructure | etcd or Consul cluster (3+ nodes for quorum) |
| Failure mode | If etcd quorum is lost, no config changes possible |

**Pros:**
- Built-in watch API eliminates polling.
- Strong consistency guarantees via Raft.
- Natural fit for distributed configuration management.

**Cons:**
- Adds a significant new infrastructure dependency (etcd cluster needs 3 nodes for HA).
- RustPBX config is relational (routes reference trunks by ID, routes have foreign keys
  to trunk IDs) -- KV stores are a poor fit for relational data.
- Duplicates the database: config would exist in both etcd and PostgreSQL, requiring
  bidirectional sync.
- Overkill for a 2-8 node SIP cluster.

### 2.4 File-Based with rsync

Each node reads config from local TOML files. A central file server or rsync cron job
pushes file changes to all nodes. Nodes watch for file modifications via `inotify` or
polling.

| Attribute | Assessment |
|-----------|------------|
| Consistency | Weak -- file propagation has variable delay, partial writes possible |
| Latency | Seconds to minutes depending on rsync interval |
| Complexity | Low (if already using file-based config), high for automation |
| Infrastructure | Shared filesystem or rsync/scp infrastructure |
| Failure mode | Network partition leaves nodes with stale files |

**Pros:**
- Works without any database or message broker.
- Simple to understand and debug (config is a readable file on disk).
- Already partially supported: `ProxyDataContext` loads from TOML file globs.

**Cons:**
- Does not integrate with the console UI (which writes to the database, not files).
- Race conditions: partial file writes can cause parse errors.
- No atomic multi-file updates (changing a trunk and its referencing route).
- Not compatible with the database-centric direction of the clustering architecture.
- Requires external orchestration (cron, Ansible, etc.) for file distribution.

### 2.5 API-Driven Config Reload

Each node exposes a reload API endpoint. When a change is made on one node, it calls
the reload endpoint on all peer nodes via HTTP.

| Attribute | Assessment |
|-----------|------------|
| Consistency | Eventual -- depends on all HTTP calls succeeding |
| Latency | Sub-second (direct HTTP call) |
| Complexity | Medium -- requires peer discovery, retry logic, circuit breaking |
| Infrastructure | None beyond existing HTTP server |
| Failure mode | If a peer is unreachable, its config is stale until next attempt |

**Pros:**
- No new infrastructure dependencies.
- Direct and explicit: the originating node knows exactly which peers it notified.
- Reload is immediate on success.

**Cons:**
- Requires peer discovery (each node must know all other nodes' HTTP addresses).
- N-1 HTTP calls per change (does not scale well beyond 4-5 nodes).
- Partial failure: if 2 of 3 peers acknowledge but 1 fails, config is inconsistent.
- Requires retry/backoff logic and health tracking for each peer.
- Tight coupling between nodes (every node is an HTTP client to every other node).

### 2.6 Decision Matrix

| Criterion | Weight | DB-Centric | Pub/Sub | etcd/Consul | File/rsync | API Reload |
|-----------|--------|-----------|---------|-------------|------------|------------|
| Consistency | 5 | 5 | 3 | 5 | 2 | 3 |
| Latency | 3 | 3 | 5 | 5 | 1 | 4 |
| Simplicity | 4 | 5 | 3 | 1 | 3 | 3 |
| Infra cost | 4 | 5 | 4 | 1 | 4 | 5 |
| Failure tolerance | 4 | 4 | 2 | 4 | 2 | 2 |
| **Weighted total** | | **90** | **67** | **63** | **49** | **68** |

**Selected approach: Database-Centric Config with optional pub/sub notification.**

---

## 3. Recommended Design: Database-Centric Config

### 3.1 Architecture Overview

```
     Admin Console / REST API
              |
              v
     +--------+--------+
     | Node 1 (writer) |
     | 1. Write to DB  |
     | 2. Bump version |
     | 3. Reload local |
     | 4. Notify peers  |
     +--------+--------+
              |
    +---------+---------+
    |                   |
    v                   v
+---+---+    +----------+----------+
| Postgres|    | Redis pub/sub       |
| (truth) |    | cluster:config      |
+---------+    +----------+----------+
                          |
              +-----------+-----------+
              |                       |
              v                       v
     +--------+--------+    +--------+--------+
     | Node 2           |    | Node 3           |
     | 1. Recv notify   |    | 1. Recv notify   |
     | 2. Check version |    | 2. Check version |
     | 3. Reload from DB|    | 3. Reload from DB|
     +-----------------+    +-----------------+
```

### 3.2 Config Version Counter

Introduce a `rustpbx_config_meta` table (or a simpler approach: a single-row
`config_version` table) that tracks the global configuration version.

```sql
CREATE TABLE IF NOT EXISTS rustpbx_config_meta (
    section     VARCHAR(64) PRIMARY KEY,   -- 'routes', 'trunks', 'acl', 'extensions', 'queues'
    version     BIGINT NOT NULL DEFAULT 0,
    updated_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_by  VARCHAR(128)               -- node_id of the writer
);

-- Seed rows
INSERT INTO rustpbx_config_meta (section, version) VALUES
    ('routes', 0),
    ('trunks', 0),
    ('acl', 0),
    ('extensions', 0),
    ('queues', 0);
```

When any config write occurs (insert, update, delete on `rustpbx_routes`,
`rustpbx_sip_trunks`, etc.), the API handler also increments the corresponding
version:

```sql
UPDATE rustpbx_config_meta
SET version = version + 1, updated_at = NOW(), updated_by = 'node1'
WHERE section = 'routes';
```

This is done within the same database transaction as the config change itself,
ensuring atomicity.

### 3.3 Node-Local Version Tracking

Each node maintains a local `HashMap<String, i64>` mapping section names to the
last-known version. On startup, the node loads all config from the database and
records the current versions. On each poll cycle (or notification), it compares
local versions against the database and reloads only the sections that changed.

```rust
// Conceptual addition to ProxyDataContext or a new ConfigSyncManager
struct ConfigSyncState {
    known_versions: HashMap<String, i64>,  // section -> version
    last_check: Instant,
}
```

### 3.4 Reload Flow

When a config change is detected (either via notification or polling):

1. Query `rustpbx_config_meta` for all sections.
2. Compare each section's `version` against the locally known version.
3. For each changed section, call the corresponding reload method:
   - `"routes"` -> `data_context.reload_routes(true, None)`
   - `"trunks"` -> `data_context.reload_trunks(true, None)` + re-evaluate trunk registrations
   - `"extensions"` -> invalidate `ExtensionUserBackend` LRU cache
   - `"acl"` -> `data_context.reload_acl_rules(false, None)`
   - `"queues"` -> `data_context.reload_queues(false, None)`
4. Update the local `known_versions` map.
5. Log the reload with metrics (section, old version, new version, duration).

---

## 4. Change Notification Optimization

### 4.1 Baseline: Polling with Version Counter

The simplest notification mechanism is periodic polling. Each node runs a background
task:

```rust
async fn config_poll_loop(
    data_context: Arc<ProxyDataContext>,
    db: DatabaseConnection,
    poll_interval: Duration,       // default: 5 seconds
    cancel_token: CancellationToken,
) {
    let mut known_versions: HashMap<String, i64> = HashMap::new();
    // Load initial versions
    load_current_versions(&db, &mut known_versions).await;

    loop {
        tokio::select! {
            _ = tokio::time::sleep(poll_interval) => {
                if let Some(changed) = check_for_changes(&db, &known_versions).await {
                    for section in changed {
                        reload_section(&data_context, &db, &section).await;
                        // Update known_versions after successful reload
                    }
                }
            }
            _ = cancel_token.cancelled() => break,
        }
    }
}
```

**Characteristics:**
- Worst-case propagation delay: `poll_interval` (5 seconds default).
- Average propagation delay: `poll_interval / 2` (2.5 seconds).
- Database load: one lightweight SELECT per poll per node (reading 5 rows from
  `rustpbx_config_meta`).
- No additional infrastructure required.

### 4.2 PostgreSQL LISTEN/NOTIFY

PostgreSQL provides a built-in pub/sub mechanism via `LISTEN`/`NOTIFY`. This
eliminates polling latency without adding Redis as a dependency.

On config change (in the API handler or via a database trigger):

```sql
-- After updating rustpbx_config_meta
NOTIFY config_changed, 'routes';
```

Each node's background task:

```sql
LISTEN config_changed;
```

When a notification arrives, the node checks the version counter and reloads if
needed.

**Characteristics:**
- Propagation delay: sub-second (typically < 100ms).
- Requires a dedicated PostgreSQL connection per node held open for LISTEN.
- Notifications are transient: if the connection drops, missed notifications are
  lost. The polling fallback handles this.
- Only works with PostgreSQL (not MySQL/SQLite). Since clustering requires
  PostgreSQL anyway, this is acceptable.

**Implementation note:** The `sqlx` crate (already a dependency via `user_db.rs`)
supports PostgreSQL LISTEN/NOTIFY via `PgListener`. The SeaORM connection cannot
be used directly for LISTEN; a separate `sqlx::PgPool` connection is needed.

### 4.3 Redis Pub/Sub (Optional Enhancement)

If Redis is already deployed for registration caching and cluster heartbeat (per
the clustering architecture design), it can also carry config change notifications.

On config change:

```
PUBLISH cluster:config_reload '{"section":"routes","version":42,"node":"node1"}'
```

Each node subscribes:

```
SUBSCRIBE cluster:config_reload
```

**Characteristics:**
- Propagation delay: sub-second.
- Fire-and-forget: missed messages require polling fallback.
- Adds no new infrastructure if Redis is already in use.
- Works regardless of database backend (PostgreSQL, MySQL).

### 4.4 Recommended Notification Strategy

Use a layered approach:

1. **Primary (PostgreSQL deployments):** `LISTEN`/`NOTIFY` for instant propagation.
2. **Primary (MySQL deployments):** Redis pub/sub if Redis is available, otherwise
   polling only.
3. **Fallback (always):** Polling with version counter every 5-10 seconds. This
   guarantees convergence even if the notification channel fails.

The notification is a *hint* that triggers an immediate version check. The version
counter in `rustpbx_config_meta` is the authoritative mechanism -- notifications
only reduce latency.

---

## 5. Config Sections and Reload Strategy

Each configuration area has different reload characteristics and side effects.

### 5.1 Routes

| Property | Value |
|----------|-------|
| DB table | `rustpbx_routes` |
| Runtime structure | `ProxyDataContext::routes: RwLock<Vec<RouteRule>>` |
| Reload method | `reload_routes(true, None)` |
| Side effects | None -- routes are stateless pattern matchers |
| Impact on active calls | None -- in-flight calls already have their route resolved |

**Reload procedure:**
1. Call `load_routes_from_db()` to fetch active routes from PostgreSQL.
2. Merge with any file-based route overrides (from `routes_files` config).
3. Sort by priority.
4. Acquire write lock on `self.routes`, swap in the new `Vec<RouteRule>`.
5. Release write lock.

New calls will use the updated routes. Active calls are unaffected because the route
was resolved at INVITE time and stored in the `CallContext`.

### 5.2 Trunks

| Property | Value |
|----------|-------|
| DB table | `rustpbx_sip_trunks` |
| Runtime structure | `ProxyDataContext::trunks: RwLock<HashMap<String, TrunkConfig>>` |
| Reload method | `reload_trunks(true, None)` |
| Side effects | May require re-registration with upstream providers |
| Impact on active calls | None for existing calls; new calls use updated trunks |

**Reload procedure:**
1. Call `load_trunks_from_db()` to fetch active trunks from PostgreSQL.
2. Merge with file-based trunk overrides.
3. Acquire write lock on `self.trunks`, swap in the new `HashMap`.
4. Release write lock.
5. **Post-reload:** Compare old and new trunk sets. For any trunk where `register = true`
   and credentials or destination changed, signal the `TrunkRegistrationModule` to
   restart the registration loop for that trunk. For removed trunks, send an
   un-REGISTER (expires=0).

**Trunk registration coordination:** In a cluster, only one node (the trunk registration
leader, elected via Redis distributed lock per rpbx-mwi.6) performs upstream REGISTER.
When trunk config changes, the leader must be notified to re-register. This is handled
by the same config change notification mechanism -- the leader node's reload detects the
trunk change and restarts the affected registration loops.

### 5.3 Extensions / Users

| Property | Value |
|----------|-------|
| DB table | `rustpbx_extensions` |
| Runtime structure | `ExtensionUserBackend` with `LruCache` (10,000 entries, TTL-based) |
| Reload method | Cache invalidation (flush or targeted eviction) |
| Side effects | None -- next auth request fetches fresh data from DB |
| Impact on active calls | None -- user lookup occurs only at REGISTER/INVITE time |

**Reload procedure:**
1. On version change for `"extensions"`, flush the LRU cache in `ExtensionUserBackend`.
2. The cache has a configurable TTL (constructor parameter `ttl_secs`). Even without
   explicit flush, stale entries expire naturally.
3. For targeted changes (single extension modified), a more efficient approach is to
   evict only the affected cache key. This requires the notification payload to include
   the changed extension ID or username.

**Cache flush implementation:**
```rust
// Add to ExtensionUserBackend
pub fn invalidate_cache(&self) {
    self.cache.lock().unwrap().clear();
}

pub fn invalidate_user(&self, username: &str, realm: Option<&str>) {
    self.cache.lock().unwrap().pop(&(username.to_string(), realm.map(|r| r.to_string())));
}
```

### 5.4 ACL Rules

| Property | Value |
|----------|-------|
| DB table | None currently (file-based) |
| Runtime structure | `ProxyDataContext::acl_rules: RwLock<Vec<String>>` |
| Reload method | `reload_acl_rules(false, None)` |
| Side effects | Immediate effect on new SIP transactions |
| Impact on active calls | None -- ACL is checked only on transaction begin |

**Migration needed:** To support database-backed ACL rules in a cluster, add a
`rustpbx_acl_rules` table:

```sql
CREATE TABLE IF NOT EXISTS rustpbx_acl_rules (
    id          BIGSERIAL PRIMARY KEY,
    rule        VARCHAR(256) NOT NULL,       -- e.g., "allow 10.0.0.0/8"
    priority    INTEGER NOT NULL DEFAULT 100,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    description VARCHAR(256),
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Reload procedure:**
1. Load ACL rules from database (ordered by priority).
2. Merge with file-based ACL rules (local overrides take precedence for node-specific
   rules like "allow <this-node's-management-IP>").
3. If no rules exist, apply the default: `["allow all", "deny all"]`.
4. Acquire write lock on `self.acl_rules`, swap in new rules.
5. Release write lock.

### 5.5 Recording Policies

| Property | Value |
|----------|-------|
| DB table | Embedded in `rustpbx_sip_trunks.metadata` and route `metadata` JSON |
| Runtime structure | Per-trunk `RecordingPolicy` field in `TrunkConfig` |
| Reload method | Reloaded as part of trunk/route reload |
| Side effects | None for active calls |
| Impact on active calls | None -- recording decision is made at call setup |

Recording policies are not a separate config section. They are embedded in trunk and
route metadata and are reloaded automatically when trunks or routes are reloaded. No
separate version tracking is needed.

### 5.6 Queues

| Property | Value |
|----------|-------|
| DB table | None currently (file-based + generated TOML) |
| Runtime structure | `ProxyDataContext::queues: RwLock<HashMap<String, RouteQueueConfig>>` |
| Reload method | `reload_queues(false, None)` |
| Side effects | Active queue sessions may reference stale config |
| Impact on active calls | Calls already in a queue continue with old config |

**Migration needed (future):** If queues are moved to the database, add a
`rustpbx_queues` table. For now, queue sync relies on file-based distribution
or a future database migration.

---

## 6. Zero-Downtime Updates

### 6.1 RwLock Semantics

The `ProxyDataContext` already uses `std::sync::RwLock` for each config section.
This provides the correct semantics for zero-downtime updates:

- **Readers** (in-flight call routing, ACL checks, trunk lookups): Acquire a read
  lock, clone the data (or an `Arc` to it), release the lock immediately. The
  `routes_snapshot()`, `trunks_snapshot()`, and `acl_rules_snapshot()` methods
  already follow this pattern.

- **Writer** (config reload): Acquires a write lock, replaces the entire data
  structure, releases the write lock. This blocks readers only for the duration of
  the pointer swap (nanoseconds for an `Arc` swap, microseconds for a `HashMap`
  clone).

### 6.2 Atomic Swap Pattern

The current implementation already follows the atomic swap pattern correctly:

```rust
// From data.rs -- trunk reload
let len = trunks.len();
*self.trunks.write().unwrap() = trunks;  // Atomic swap
```

The new trunks `HashMap` is fully constructed before the write lock is acquired.
The write lock is held only for the assignment (pointer swap). This ensures:

1. Readers never see a partially-constructed config.
2. The write lock duration is bounded and predictable (microseconds).
3. No reader starvation: `std::sync::RwLock` is fair on most platforms.

### 6.3 New Calls vs Active Calls

Config changes apply only to new calls. This is enforced by the call architecture:

- At INVITE time, the `CallModule` reads the current route and trunk config,
  resolves the destination, and stores the result in `CallContext` (which is
  immutable for the call's lifetime).
- `MediaBridge` parameters (codec, recording policy) are set at call setup and
  do not change mid-call.
- In-dialog requests (re-INVITE, BYE) are processed using the `CallSession`'s
  stored context, not the current `ProxyDataContext`.

**Exception:** If a call is in the ringing phase (INVITE sent but no final
response), a trunk reload that removes the target trunk could cause the call to
fail. This is acceptable -- the call would have failed anyway if the trunk was
removed. The caller hears a 503 Service Unavailable or 480 Temporarily Unavailable.

### 6.4 Reload Ordering

When multiple sections change simultaneously (e.g., a trunk is added and a route
referencing it is created), reload ordering matters:

1. **Trunks first:** Routes reference trunks by name. If a route is loaded before
   its target trunk exists, the route will fail to resolve at call time (but will
   not crash -- it returns a routing error to the caller).
2. **Routes second:** After trunks are loaded, routes can resolve trunk references.
3. **ACL rules, extensions, queues:** Order-independent.

The reload loop should enforce this ordering:

```rust
let changed_sections = detect_changes(&db, &known_versions).await;
if changed_sections.contains("trunks") {
    reload_trunks().await;
}
if changed_sections.contains("routes") {
    reload_routes().await;
}
// Remaining sections in any order
for section in &changed_sections {
    match section.as_str() {
        "trunks" | "routes" => {}, // Already handled
        "extensions" => invalidate_extension_cache(),
        "acl" => reload_acl_rules(),
        "queues" => reload_queues(),
        _ => {},
    }
}
```

---

## 7. Conflict Resolution

### 7.1 Concurrent Writes

In a cluster, two administrators could modify the same route on different nodes at
the same time. Both nodes write to the shared PostgreSQL database. PostgreSQL's MVCC
and row-level locking prevent data corruption, but the business-level conflict must
be addressed.

### 7.2 Last-Write-Wins (Default)

The simplest strategy is last-write-wins, using the `updated_at` timestamp as the
tiebreaker. This is already the implicit behavior with PostgreSQL:

- Admin A on Node 1 updates route "outbound-pstn" at T=100.
- Admin B on Node 2 updates the same route at T=101.
- The database contains Admin B's version (last write).
- When both nodes reload, they converge on Admin B's version.

No special handling is required. The `updated_at` column on `rustpbx_routes` and
`rustpbx_sip_trunks` is already set on every UPDATE.

**Trade-off:** Admin A's change is silently overwritten. For a small team managing
a PBX, this is usually acceptable. The console UI can show a warning if the
`updated_at` timestamp changed between load and save (see optimistic locking below).

### 7.3 Optimistic Locking (Optional Enhancement)

For deployments where config conflicts are a concern, add optimistic locking using
a `version` column on each config table:

```sql
ALTER TABLE rustpbx_routes ADD COLUMN row_version BIGINT NOT NULL DEFAULT 0;
ALTER TABLE rustpbx_sip_trunks ADD COLUMN row_version BIGINT NOT NULL DEFAULT 0;
```

The update operation includes the expected version:

```sql
UPDATE rustpbx_routes
SET name = $1, priority = $2, ..., row_version = row_version + 1, updated_at = NOW()
WHERE id = $3 AND row_version = $4;
```

If `row_version` has changed since the admin loaded the form, the UPDATE affects
zero rows. The API returns `409 Conflict` and the admin must reload and re-apply
their changes.

**Console UI flow:**
1. Admin loads route edit form. The form includes a hidden `row_version` field.
2. Admin modifies fields and submits.
3. Server attempts UPDATE with the original `row_version`.
4. If successful (1 row affected): change is applied, version incremented.
5. If failed (0 rows affected): return 409 with message "This route was modified
   by another administrator. Please reload and try again."

### 7.4 Conflict Scope

Not all config sections need optimistic locking:

| Section | Conflict risk | Recommended strategy |
|---------|--------------|---------------------|
| Routes | Medium (multiple admins may edit routing rules) | Optimistic locking |
| Trunks | Low (trunk config changes are infrequent) | Last-write-wins |
| Extensions | Low (typically managed by one admin) | Last-write-wins |
| ACL rules | Very low (security config rarely changes) | Last-write-wins |
| Recording | Very low (embedded in trunk/route metadata) | Inherits from parent |

---

## 8. Implementation Plan

### 8.1 Phase 1: Database Version Tracking (No New Dependencies)

**Estimated effort:** 2-3 days

1. Create `rustpbx_config_meta` table via SeaORM migration.
2. Add version bump logic to the console API handlers for routes, trunks, and
   extensions (in `src/console/handlers/setting.rs`).
3. Add a `ConfigSyncManager` struct that:
   - Holds a `HashMap<String, i64>` of known versions.
   - Provides `check_for_changes()` and `reload_section()` methods.
4. Spawn a polling background task in `src/app.rs` (or the SipServer builder) that
   calls `ConfigSyncManager::poll()` every N seconds.
5. Wire reload to existing `ProxyDataContext` methods.

**Files to modify:**
- `src/models/mod.rs` -- add migration for `rustpbx_config_meta`
- `src/proxy/data.rs` -- add `ConfigSyncManager` or extend `ProxyDataContext`
- `src/console/handlers/setting.rs` -- bump version on config writes
- `src/app.rs` -- spawn poll loop

### 8.2 Phase 2: Instant Notification (PostgreSQL LISTEN/NOTIFY)

**Estimated effort:** 1-2 days

1. Add a `PgListener` connection in `ConfigSyncManager`.
2. On config write, issue `NOTIFY config_changed, '<section>'`.
3. The poll loop also listens for notifications and triggers immediate reload.
4. Retain polling as a fallback (increase interval to 30 seconds when notifications
   are working).

**Files to modify:**
- `src/proxy/data.rs` -- add LISTEN/NOTIFY handling
- `src/console/handlers/setting.rs` -- add NOTIFY after version bump

### 8.3 Phase 3: Redis Pub/Sub Integration (When Redis Is Available)

**Estimated effort:** 1 day (if Redis integration exists from rpbx-mwi.2)

1. On config write, also `PUBLISH cluster:config_reload` to Redis.
2. `ConfigSyncManager` subscribes to `cluster:config_reload`.
3. On message receipt, trigger immediate version check and reload.
4. This provides notification for MySQL deployments (which lack LISTEN/NOTIFY).

### 8.4 Phase 4: ACL Database Migration

**Estimated effort:** 1-2 days

1. Create `rustpbx_acl_rules` table.
2. Add console UI page for ACL management.
3. Modify `reload_acl_rules()` to load from database as well as files.
4. Add `"acl"` section to version tracking.

### 8.5 Phase 5: Optimistic Locking (Optional)

**Estimated effort:** 1 day

1. Add `row_version` column to `rustpbx_routes` and `rustpbx_sip_trunks`.
2. Modify update API handlers to check `row_version`.
3. Return 409 Conflict on version mismatch.
4. Update console UI to handle 409 and prompt the admin.

---

## 9. Configuration

New config section for cluster-aware config sync:

```toml
[cluster.config_sync]
# Polling interval when notifications are unavailable or as fallback
poll_interval_secs = 5

# Use PostgreSQL LISTEN/NOTIFY for instant propagation (requires PostgreSQL)
pg_notify = true

# Use Redis pub/sub for instant propagation (requires [cluster.redis_url])
redis_notify = true

# Channel name for Redis pub/sub notifications
redis_channel = "cluster:config_reload"

# Sections to sync (default: all)
# sync_sections = ["routes", "trunks", "extensions", "acl", "queues"]
```

Node-specific settings that are never synced:

```toml
# These remain in each node's local config file
external_ip = "10.0.0.10"
rtp_start_port = 20000
rtp_end_port = 25000
node_id = "node1"
addr = "0.0.0.0"
udp_port = 5060
tcp_port = 5060
```

---

## 10. Observability

### 10.1 Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `config_sync_reloads_total` | Counter | Total reloads by section |
| `config_sync_reload_duration_ms` | Histogram | Time to reload each section |
| `config_sync_version` | Gauge | Current known version per section |
| `config_sync_poll_errors_total` | Counter | Failed version check queries |
| `config_sync_notifications_received` | Counter | NOTIFY/pub-sub events received |
| `config_sync_lag_seconds` | Gauge | Time since last successful sync |

### 10.2 Logging

All reload events are logged at INFO level with structured fields:

```
INFO config_sync: section="routes" old_version=41 new_version=42
     node="node1" trigger="pg_notify" duration_ms=12 routes_loaded=8
```

Errors (failed DB queries, parse failures) are logged at WARN/ERROR:

```
WARN config_sync: section="trunks" error="connection refused"
     node="node1" retry_in_secs=5
```

### 10.3 Admin API Endpoints

```
GET /admin/config/versions
    -> {"routes": 42, "trunks": 15, "extensions": 8, "acl": 3, "queues": 1}

POST /admin/config/reload?section=routes
    -> {"reloaded": "routes", "old_version": 41, "new_version": 42, "duration_ms": 12}

POST /admin/config/reload
    -> {"reloaded": ["routes", "trunks", "acl", "extensions", "queues"], ...}
```

---

## 11. Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| PostgreSQL outage blocks config changes | High -- no new config can be written | All nodes retain last-known config and continue operating. Alert on sync lag. |
| Notification channel (NOTIFY/Redis) fails | Low -- polling fallback ensures convergence within poll_interval | Polling is always active as a fallback. |
| Large config reload causes write lock contention | Low -- write lock is held for microseconds | Pre-build new data structures before acquiring lock. Monitor `reload_duration_ms`. |
| Trunk re-registration storm after bulk config change | Medium -- many trunks restarting registration simultaneously | Stagger re-registration with jitter. Only re-register trunks whose credentials or dest actually changed. |
| Admin A's changes silently overwritten by Admin B | Low -- small team typical for PBX admin | Optimistic locking (Phase 5) prevents silent overwrites. Audit log tracks all changes. |
| ACL rules loaded from DB differ from file-based rules | Medium -- security implications | File-based ACL rules take priority (merge strategy: file rules first, then DB rules). Log merge result. |
