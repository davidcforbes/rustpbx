# Trunk Registration Coordination Design

## Document Metadata

| Field | Value |
|-------|-------|
| Task | rpbx-mwi.6 |
| Status | Draft |
| Created | 2026-02-24 |
| Depends On | rpbx-mwi.1 (SIP LB), rpbx-mwi.2 (shared state) |

---

## 1. Problem Statement

RustPBX supports SIP trunk registration for inbound call delivery. The
`TrunkRegistrationModule` (`src/proxy/trunk_register.rs`) iterates over all
trunks with `register = true`, then spawns a perpetual `register_loop` per
trunk that sends REGISTER to the upstream provider (e.g., Telnyx) and
re-registers at 85% of the granted expiry interval.

This works correctly for a single node. In a multi-node cluster it breaks in
three ways:

1. **Duplicate registrations.** Every node starts its own `register_loop` for
   every trunk. The provider receives N concurrent REGISTER streams for the
   same credential. Most SIP registrars accept the latest Contact, so the
   nodes fight over which Contact is authoritative. The last REGISTER to
   arrive "wins," but a few seconds later another node overwrites it. Inbound
   INVITEs from the provider land on whichever node happened to register last,
   creating unpredictable routing.

2. **Missed inbound calls on failover.** If the node that currently "owns" the
   registration crashes, the registration remains active at the provider until
   its expiry timer fires (typically 3600 seconds). During that window the
   provider sends inbound INVITEs to the dead node's Contact address. All
   inbound calls for that trunk are black-holed until the registration
   naturally expires, which could be up to an hour.

3. **Registration storms after restart.** When a failed node recovers, it
   immediately sends REGISTER for all trunks, competing again with the node
   that took over. This causes a brief routing oscillation and can trigger
   rate-limiting or 429 responses from the provider.

The goal of this design is to ensure that exactly one node at a time is
responsible for each trunk's registration, that ownership transfers quickly
on failure, and that trunk assignments are balanced across the cluster.

---

## 2. Leader Election for Trunk Ownership

Each trunk needs a single owner at any given time. This is a classic
distributed lease/lock problem. Three options were evaluated:

| Approach | Pros | Cons |
|----------|------|------|
| **Database-backed lease** (UPDATE ... WHERE) | No new dependency; PostgreSQL already required for clustering | Higher latency (~2-10ms per lease op); row-level locking needed |
| **Redis-based lock** (SET NX EX) | Sub-millisecond; natural TTL expiry | Adds Redis as a hard dependency for trunk registration |
| **etcd lease** | Built for distributed coordination; watch support | Heavy new dependency; overkill for 2-8 node clusters |

**Recommendation: Database-backed lease.** The clustering architecture
(see `clustering-architecture.md` Section 3.2) already requires PostgreSQL
as the shared persistent store. Adding a `trunk_registration_leases` table
avoids introducing Redis as a hard dependency for this subsystem alone. The
lease renewal interval (every 30-60 seconds) generates negligible database
load -- a single UPDATE per trunk per interval.

If Redis is available (Phase 2 of the clustering rollout), the coordinator
can optionally use Redis `SET NX EX` instead, reducing lease-check latency.
The design supports both backends through a `LeaseBackend` trait.

---

## 3. Registration Lease Table

### 3.1 Schema

```sql
CREATE TABLE rustpbx_trunk_registration_leases (
    trunk_id        VARCHAR(120) PRIMARY KEY,  -- matches TrunkConfig key / sip_trunk.name
    owner_node_id   VARCHAR(120) NOT NULL,
    lease_acquired_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    lease_expires_at   TIMESTAMP NOT NULL,
    last_register_at   TIMESTAMP,
    register_status    VARCHAR(32) NOT NULL DEFAULT 'pending',
    register_expires   INTEGER NOT NULL DEFAULT 3600,
    failure_count      INTEGER NOT NULL DEFAULT 0,
    last_error         TEXT
);

CREATE INDEX idx_trunk_lease_expires ON rustpbx_trunk_registration_leases (lease_expires_at);
CREATE INDEX idx_trunk_lease_owner   ON rustpbx_trunk_registration_leases (owner_node_id);
```

### 3.2 Column Semantics

| Column | Purpose |
|--------|---------|
| `trunk_id` | The trunk name as it appears in `[proxy.trunks.<name>]` config or `rustpbx_sip_trunks.name`. |
| `owner_node_id` | The `node_id` from `[cluster]` config of the node that holds this lease. |
| `lease_acquired_at` | When this node first claimed or last renewed the lease. |
| `lease_expires_at` | Hard deadline. If not renewed by this time, any node may claim the trunk. |
| `last_register_at` | Timestamp of the last successful REGISTER sent to the provider. |
| `register_status` | One of: `pending`, `registered`, `failed`, `expired`. |
| `register_expires` | The expiry value granted by the provider in the 200 OK. |
| `failure_count` | Consecutive REGISTER failures. Used for health monitoring and automatic release. |
| `last_error` | Human-readable description of the last failure (e.g., "401 Unauthorized", "timeout"). |

### 3.3 Lease Operations

**Claim (atomic conditional insert/update):**

```sql
-- Try to insert a new lease (trunk not yet claimed by anyone)
INSERT INTO rustpbx_trunk_registration_leases
    (trunk_id, owner_node_id, lease_acquired_at, lease_expires_at, register_status)
VALUES ($1, $2, NOW(), NOW() + INTERVAL '90 seconds', 'pending')
ON CONFLICT (trunk_id) DO UPDATE
SET owner_node_id = $2,
    lease_acquired_at = NOW(),
    lease_expires_at = NOW() + INTERVAL '90 seconds',
    register_status = 'pending',
    failure_count = 0,
    last_error = NULL
WHERE rustpbx_trunk_registration_leases.lease_expires_at < NOW();
```

The `WHERE lease_expires_at < NOW()` clause ensures that a claim only
succeeds if the trunk is unclaimed or the previous lease has expired. This
is the atomic compare-and-swap that prevents two nodes from simultaneously
claiming the same trunk.

**Renew:**

```sql
UPDATE rustpbx_trunk_registration_leases
SET lease_expires_at = NOW() + INTERVAL '90 seconds',
    lease_acquired_at = NOW()
WHERE trunk_id = $1
  AND owner_node_id = $2
  AND lease_expires_at > NOW();
```

Returns affected rows = 1 on success, 0 if the lease was lost (expired or
stolen). The owning node must check the return value and stop its
`register_loop` if the lease was lost.

**Release (graceful shutdown):**

```sql
DELETE FROM rustpbx_trunk_registration_leases
WHERE trunk_id = $1 AND owner_node_id = $2;
```

On graceful shutdown, the node deletes its leases so that other nodes can
claim them immediately without waiting for expiry.

**Update registration status:**

```sql
UPDATE rustpbx_trunk_registration_leases
SET last_register_at = NOW(),
    register_status = $3,
    register_expires = $4,
    failure_count = CASE WHEN $3 = 'registered' THEN 0 ELSE failure_count + 1 END,
    last_error = $5
WHERE trunk_id = $1 AND owner_node_id = $2;
```

---

## 4. Failover Handoff

### 4.1 Timing Analysis

Three timers interact during failover:

| Timer | Value | Controlled By |
|-------|-------|---------------|
| Provider registration expiry | 3600s (typical) | Upstream provider's 200 OK Expires header |
| Lease TTL | 90s | RustPBX cluster config |
| Lease renewal interval | 30s | RustPBX cluster config |
| Node heartbeat timeout | 30s | Cluster health monitor |

**Worst-case failover gap:** When the owning node crashes, its lease expires
after at most 90 seconds (the lease TTL). The scanning node detects the
expired lease on its next scan cycle (every 15 seconds). Total worst case:
90 + 15 = 105 seconds.

**Typical failover gap:** If the node was renewing every 30 seconds and just
renewed before crashing, the lease expires in ~90 seconds. The scanner
detects it within 15 seconds of expiry. Typical gap: ~90-105 seconds.

**Why 90 seconds is acceptable:** The upstream provider still has a valid
registration (expiry 3600s). Inbound calls during the failover gap are sent
to the dead node's Contact. They will fail (timeout/503), and the provider
may retry or return a busy signal. The new owner sends REGISTER immediately
upon claiming the lease, updating the Contact within seconds of claiming.
The true blackout window is the lease expiry time, not the registration
expiry time.

### 4.2 Failover Sequence

```
Time 0s    Node A crashes (held lease for trunk "telnyx")
           Node A's lease_expires_at = NOW() + 60s (renewed 30s ago)

Time 0-60s Lease still valid. Node B's scanner sees the lease is not expired.
           Inbound calls to the trunk hit dead Node A. Provider gets timeouts.

Time 60s   Lease expires.

Time 60-75s Node B's scanner (runs every 15s) detects expired lease.
           Node B executes claim query: INSERT ... ON CONFLICT ... WHERE expires < NOW()
           Claim succeeds (affected rows = 1).

Time 75s   Node B's TrunkCoordinator spawns register_loop for "telnyx".
           Sends REGISTER to provider.

Time 76s   Provider returns 200 OK. Contact now points to Node B.
           Inbound calls resume on Node B.

Total blackout: ~75 seconds (typical)
```

### 4.3 Reducing the Gap

To reduce the failover gap below 90 seconds:

1. **Shorter lease TTL (30s) with more frequent renewal (10s).** Increases
   database write load (one UPDATE per trunk every 10 seconds) but cuts
   the worst-case gap to ~45 seconds. Acceptable for clusters with fewer
   than 50 trunks.

2. **Active failure detection via heartbeat.** If the cluster heartbeat
   monitor (Section 3.5.4 in clustering-architecture.md) detects that Node A
   is dead (heartbeat timeout = 30s), it can proactively expire Node A's
   leases rather than waiting for the TTL. This reduces the gap to the
   heartbeat timeout (~30 seconds) plus scan interval (~15 seconds) = ~45
   seconds.

3. **Combination approach (recommended).** Use 60-second lease TTL with
   20-second renewal. On heartbeat-detected node failure, immediately expire
   the dead node's leases:

   ```sql
   UPDATE rustpbx_trunk_registration_leases
   SET lease_expires_at = NOW()
   WHERE owner_node_id = $dead_node_id;
   ```

   This brings typical failover to ~20-35 seconds.

---

## 5. Multi-Trunk Load Distribution

### 5.1 Problem

With N trunks and M nodes in the cluster, trunk registrations should be
distributed evenly so that no single node handles all upstream REGISTER
traffic and inbound call routing.

### 5.2 Assignment Algorithm

Use consistent hashing for deterministic assignment with minimal disruption
when nodes join or leave:

```
preferred_node = nodes_sorted[hash(trunk_id) % len(nodes_sorted)]
```

Where `nodes_sorted` is the alphabetically sorted list of active node IDs.
This ensures all nodes agree on the assignment without coordination.

**Example with 3 trunks and 2 nodes:**

```
Trunk "telnyx"   -> hash("telnyx")   % 2 = 0 -> node1
Trunk "twilio"   -> hash("twilio")   % 2 = 1 -> node2
Trunk "bandwidth" -> hash("bandwidth") % 2 = 0 -> node1
```

### 5.3 Rebalancing on Node Join/Leave

When a node joins or leaves the cluster, the preferred assignment changes
for approximately N/M trunks (where N = total trunks, M = old node count).

**On node join:**
1. New node starts its scanner loop.
2. Scanner computes preferred assignments for all trunks.
3. For trunks assigned to itself, attempts to claim the lease.
4. The claim only succeeds if the trunk is unclaimed or the current
   owner's lease has expired. This prevents stealing active leases.
5. Existing owners continue until they detect they are no longer the
   preferred node. On their next renewal cycle, they check preferred
   assignment and voluntarily release if reassigned:

   ```rust
   if self.preferred_node(trunk_id) != self.node_id {
       self.release_trunk(trunk_id).await;
   }
   ```

**On node leave (graceful):**
1. Departing node releases all its leases (DELETE query).
2. Remaining nodes' scanners detect unclaimed trunks and claim them.

**On node leave (crash):**
1. Leases expire after TTL.
2. Heartbeat monitor may proactively expire them (see Section 4.3).
3. Remaining nodes claim expired leases.

### 5.4 Consistency Guarantee

The database lease is the authoritative owner record, not the hash
assignment. The hash is a hint for load balancing. If two nodes disagree
about the active node list (temporarily, during a join/leave transition),
the database prevents double-claiming because the `WHERE lease_expires_at <
NOW()` guard is atomic.

---

## 6. Health Monitoring

### 6.1 Per-Trunk Registration Health

The `TrunkCoordinator` tracks registration health for each trunk it owns:

| Metric | Source | Meaning |
|--------|--------|---------|
| `register_status` | Lease table | Current state: pending, registered, failed, expired |
| `failure_count` | Lease table | Consecutive failures since last success |
| `last_register_at` | Lease table | Freshness of the registration |
| `last_error` | Lease table | Most recent error text |
| `register_latency_ms` | In-memory | Round-trip time for the last REGISTER transaction |
| `register_expires` | Lease table | Granted expiry from provider |

### 6.2 Automatic Release on Persistent Failure

If a trunk fails to register `max_failure_count` times consecutively
(default: 5), the owning node releases the lease. This allows another
node to attempt registration -- useful if the failure is network-path
specific (e.g., the owning node lost connectivity to the provider but
other nodes can still reach it).

```rust
if lease.failure_count >= self.config.max_failure_count {
    warn!(trunk = %trunk_id, failures = lease.failure_count,
          "releasing trunk after persistent registration failure");
    self.release_trunk(trunk_id).await?;
}
```

### 6.3 Health Endpoint Integration

Trunk registration health is exposed through the cluster health endpoint:

```
GET /health/detail
```

```json
{
  "status": "healthy",
  "node_id": "node1",
  "trunk_registrations": {
    "telnyx": {
      "status": "registered",
      "owner": "node1",
      "last_register": "2026-02-24T10:30:00Z",
      "expires_in_secs": 3060,
      "failure_count": 0
    },
    "twilio": {
      "status": "registered",
      "owner": "node2",
      "last_register": "2026-02-24T10:29:45Z",
      "expires_in_secs": 3045,
      "failure_count": 0
    }
  }
}
```

Any node can query the lease table to report the status of all trunks
cluster-wide, regardless of which node owns each trunk.

### 6.4 Alerting Thresholds

| Condition | Severity | Action |
|-----------|----------|--------|
| `register_status = failed` for > 5 minutes | Warning | Log + health endpoint degrades to "warning" |
| `failure_count >= max_failure_count` | Error | Release trunk, log, health shows "degraded" |
| Trunk unclaimed for > 2x lease TTL | Critical | No node is registering; likely all nodes failing |
| `last_register_at` older than `register_expires` | Critical | Registration has lapsed at the provider |

---

## 7. Implementation Sketch

### 7.1 Module Structure

```
src/proxy/trunk_coordinator.rs    -- TrunkCoordinator struct and lease logic
src/proxy/trunk_register.rs       -- existing register_loop (modified to accept coordinator)
src/models/trunk_lease.rs         -- SeaORM entity for rustpbx_trunk_registration_leases
```

### 7.2 Core Structs and Traits

```rust
// src/proxy/trunk_coordinator.rs

use crate::models::trunk_lease;
use crate::proxy::server::SipServerRef;
use anyhow::Result;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

/// Configuration for the trunk coordination subsystem.
pub struct TrunkCoordinatorConfig {
    pub node_id: String,
    pub lease_ttl_secs: u64,          // default: 60
    pub renewal_interval_secs: u64,   // default: 20
    pub scan_interval_secs: u64,      // default: 15
    pub max_failure_count: u32,       // default: 5
}

/// Tracks a trunk registration that this node owns.
struct OwnedTrunk {
    trunk_id: String,
    cancel_token: CancellationToken,
    register_status: String,
    failure_count: u32,
}

/// Coordinates trunk registration ownership across the cluster.
pub struct TrunkCoordinator {
    config: TrunkCoordinatorConfig,
    db: DatabaseConnection,
    server: SipServerRef,
    owned_trunks: Arc<RwLock<HashMap<String, OwnedTrunk>>>,
    cancel_token: CancellationToken,
}

impl TrunkCoordinator {
    pub fn new(
        config: TrunkCoordinatorConfig,
        db: DatabaseConnection,
        server: SipServerRef,
        cancel_token: CancellationToken,
    ) -> Self {
        Self {
            config,
            db,
            server,
            owned_trunks: Arc::new(RwLock::new(HashMap::new())),
            cancel_token,
        }
    }

    /// Main loop: scan for claimable trunks, renew owned leases, release
    /// trunks that are no longer assigned to this node.
    pub async fn run(&self) -> Result<()> {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(
                    std::time::Duration::from_secs(self.config.scan_interval_secs)
                ) => {}
                _ = self.cancel_token.cancelled() => {
                    self.release_all().await;
                    return Ok(());
                }
            }

            if let Err(e) = self.scan_and_reconcile().await {
                tracing::error!(error = %e, "trunk coordinator scan failed");
            }
        }
    }

    /// Scan all trunks with register=true. Claim unclaimed trunks assigned
    /// to this node. Renew leases for owned trunks. Release trunks no
    /// longer assigned here.
    async fn scan_and_reconcile(&self) -> Result<()> {
        let trunks = self.server.data_context.trunks_snapshot();
        let registerable: Vec<String> = trunks
            .iter()
            .filter(|(_, t)| t.register == Some(true))
            .map(|(name, _)| name.clone())
            .collect();

        let active_nodes = self.get_active_nodes().await?;

        for trunk_id in &registerable {
            let preferred = self.preferred_node(trunk_id, &active_nodes);

            if preferred == self.config.node_id {
                // This trunk is assigned to us
                if self.owns(trunk_id).await {
                    self.renew_lease(trunk_id).await?;
                } else {
                    self.claim_trunk(trunk_id).await?;
                }
            } else {
                // This trunk is assigned elsewhere
                if self.owns(trunk_id).await {
                    self.release_trunk(trunk_id).await?;
                }
            }
        }

        // Release any owned trunks that are no longer in the config
        let owned = self.owned_trunks.read().await;
        let orphans: Vec<String> = owned
            .keys()
            .filter(|k| !registerable.contains(k))
            .cloned()
            .collect();
        drop(owned);

        for trunk_id in orphans {
            self.release_trunk(&trunk_id).await?;
        }

        Ok(())
    }

    /// Attempt to claim a trunk lease via atomic INSERT ... ON CONFLICT.
    pub async fn claim_trunk(&self, trunk_id: &str) -> Result<bool> {
        let claimed = trunk_lease::claim(
            &self.db,
            trunk_id,
            &self.config.node_id,
            self.config.lease_ttl_secs,
        )
        .await?;

        if claimed {
            tracing::info!(
                trunk = %trunk_id,
                node = %self.config.node_id,
                "claimed trunk registration lease"
            );
            self.start_register_loop(trunk_id).await?;
        }

        Ok(claimed)
    }

    /// Renew an existing lease. Returns false if the lease was lost.
    pub async fn renew_lease(&self, trunk_id: &str) -> Result<bool> {
        let renewed = trunk_lease::renew(
            &self.db,
            trunk_id,
            &self.config.node_id,
            self.config.lease_ttl_secs,
        )
        .await?;

        if !renewed {
            tracing::warn!(
                trunk = %trunk_id,
                node = %self.config.node_id,
                "lost trunk registration lease (expired or stolen)"
            );
            self.stop_register_loop(trunk_id).await;
        }

        Ok(renewed)
    }

    /// Release a trunk lease and stop its register_loop.
    pub async fn release_trunk(&self, trunk_id: &str) -> Result<()> {
        trunk_lease::release(&self.db, trunk_id, &self.config.node_id).await?;
        self.stop_register_loop(trunk_id).await;

        tracing::info!(
            trunk = %trunk_id,
            node = %self.config.node_id,
            "released trunk registration lease"
        );

        Ok(())
    }

    /// Query the lease table to find which node owns a given trunk.
    pub async fn get_owner(&self, trunk_id: &str) -> Result<Option<String>> {
        trunk_lease::get_owner(&self.db, trunk_id).await
    }

    /// Release all leases (called during graceful shutdown).
    async fn release_all(&self) {
        let owned: Vec<String> = self.owned_trunks.read().await.keys().cloned().collect();
        for trunk_id in owned {
            if let Err(e) = self.release_trunk(&trunk_id).await {
                tracing::error!(trunk = %trunk_id, error = %e, "failed to release trunk on shutdown");
            }
        }
    }

    /// Determine the preferred owner for a trunk based on consistent hashing.
    fn preferred_node(&self, trunk_id: &str, active_nodes: &[String]) -> String {
        if active_nodes.is_empty() {
            return self.config.node_id.clone();
        }
        let hash = Self::hash_trunk_id(trunk_id);
        let idx = (hash as usize) % active_nodes.len();
        active_nodes[idx].clone()
    }

    fn hash_trunk_id(trunk_id: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        trunk_id.hash(&mut hasher);
        hasher.finish()
    }

    /// Check if this node currently owns the given trunk.
    async fn owns(&self, trunk_id: &str) -> bool {
        self.owned_trunks.read().await.contains_key(trunk_id)
    }

    /// Spawn a register_loop for a newly claimed trunk.
    async fn start_register_loop(&self, trunk_id: &str) -> Result<()> {
        let trunks = self.server.data_context.trunks_snapshot();
        let trunk = match trunks.get(trunk_id) {
            Some(t) => t.clone(),
            None => return Ok(()),
        };

        let token = self.cancel_token.child_token();
        let owned = OwnedTrunk {
            trunk_id: trunk_id.to_string(),
            cancel_token: token.clone(),
            register_status: "pending".to_string(),
            failure_count: 0,
        };

        self.owned_trunks
            .write()
            .await
            .insert(trunk_id.to_string(), owned);

        // Spawn the existing register_loop with the child token
        let ep = self.server.endpoint.inner.clone();
        let db = self.db.clone();
        let node_id = self.config.node_id.clone();
        let trunk_name = trunk_id.to_string();

        crate::utils::spawn(async move {
            // Delegate to the existing register_loop from trunk_register.rs,
            // extended with lease status callbacks to update the DB.
            coordinated_register_loop(
                ep, db, node_id, trunk_name, trunk, token,
            )
            .await;
        });

        Ok(())
    }

    /// Stop the register_loop for a trunk being released.
    async fn stop_register_loop(&self, trunk_id: &str) {
        if let Some(owned) = self.owned_trunks.write().await.remove(trunk_id) {
            owned.cancel_token.cancel();
        }
    }

    /// Get the list of active nodes from the heartbeat table or lease table.
    async fn get_active_nodes(&self) -> Result<Vec<String>> {
        trunk_lease::get_active_nodes(&self.db).await
    }

    /// Get registration health for all trunks (for /health/detail endpoint).
    pub async fn get_health(&self) -> Result<Vec<TrunkRegistrationHealth>> {
        trunk_lease::get_all_leases(&self.db).await
    }
}

/// Health status for a single trunk registration, returned by the health endpoint.
pub struct TrunkRegistrationHealth {
    pub trunk_id: String,
    pub owner_node_id: String,
    pub register_status: String,
    pub last_register_at: Option<chrono::DateTime<chrono::Utc>>,
    pub register_expires: i32,
    pub failure_count: i32,
    pub last_error: Option<String>,
}
```

### 7.3 SeaORM Entity Sketch

```rust
// src/models/trunk_lease.rs

use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait, Statement};
use anyhow::Result;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rustpbx_trunk_registration_leases")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub trunk_id: String,
    pub owner_node_id: String,
    pub lease_acquired_at: DateTimeUtc,
    pub lease_expires_at: DateTimeUtc,
    pub last_register_at: Option<DateTimeUtc>,
    pub register_status: String,
    pub register_expires: i32,
    pub failure_count: i32,
    pub last_error: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// Attempt to claim a trunk lease. Returns true if the claim succeeded.
pub async fn claim(
    db: &DatabaseConnection,
    trunk_id: &str,
    node_id: &str,
    ttl_secs: u64,
) -> Result<bool> {
    // Use raw SQL for the atomic INSERT ... ON CONFLICT ... WHERE pattern
    // because SeaORM doesn't directly support conditional ON CONFLICT.
    let result = db
        .execute(Statement::from_sql_and_values(
            db.get_database_backend(),
            r#"
            INSERT INTO rustpbx_trunk_registration_leases
                (trunk_id, owner_node_id, lease_acquired_at, lease_expires_at,
                 register_status, register_expires, failure_count)
            VALUES ($1, $2, CURRENT_TIMESTAMP,
                    CURRENT_TIMESTAMP + INTERVAL '1 second' * $3,
                    'pending', 3600, 0)
            ON CONFLICT (trunk_id) DO UPDATE
            SET owner_node_id = $2,
                lease_acquired_at = CURRENT_TIMESTAMP,
                lease_expires_at = CURRENT_TIMESTAMP + INTERVAL '1 second' * $3,
                register_status = 'pending',
                failure_count = 0,
                last_error = NULL
            WHERE rustpbx_trunk_registration_leases.lease_expires_at
                  < CURRENT_TIMESTAMP
            "#,
            [trunk_id.into(), node_id.into(), (ttl_secs as i64).into()],
        ))
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Renew a lease. Returns true if renewal succeeded.
pub async fn renew(
    db: &DatabaseConnection,
    trunk_id: &str,
    node_id: &str,
    ttl_secs: u64,
) -> Result<bool> {
    let result = db
        .execute(Statement::from_sql_and_values(
            db.get_database_backend(),
            r#"
            UPDATE rustpbx_trunk_registration_leases
            SET lease_expires_at = CURRENT_TIMESTAMP + INTERVAL '1 second' * $3,
                lease_acquired_at = CURRENT_TIMESTAMP
            WHERE trunk_id = $1
              AND owner_node_id = $2
              AND lease_expires_at > CURRENT_TIMESTAMP
            "#,
            [trunk_id.into(), node_id.into(), (ttl_secs as i64).into()],
        ))
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Release a lease (graceful shutdown).
pub async fn release(
    db: &DatabaseConnection,
    trunk_id: &str,
    node_id: &str,
) -> Result<()> {
    db.execute(Statement::from_sql_and_values(
        db.get_database_backend(),
        r#"
        DELETE FROM rustpbx_trunk_registration_leases
        WHERE trunk_id = $1 AND owner_node_id = $2
        "#,
        [trunk_id.into(), node_id.into()],
    ))
    .await?;

    Ok(())
}

/// Get the current owner of a trunk (if lease is still valid).
pub async fn get_owner(
    db: &DatabaseConnection,
    trunk_id: &str,
) -> Result<Option<String>> {
    let result = Entity::find_by_id(trunk_id.to_string())
        .one(db)
        .await?;

    match result {
        Some(model) if model.lease_expires_at > chrono::Utc::now() => {
            Ok(Some(model.owner_node_id))
        }
        _ => Ok(None),
    }
}
```

### 7.4 Integration with Existing TrunkRegistrationModule

The existing `TrunkRegistrationModule` in `src/proxy/trunk_register.rs`
is modified to delegate to `TrunkCoordinator` when clustering is enabled:

```rust
// Modified on_start() in TrunkRegistrationModule:

async fn on_start(&mut self) -> Result<()> {
    if let Some(coordinator) = &self.coordinator {
        // Clustering mode: coordinator manages ownership and spawns
        // register_loops only for trunks assigned to this node.
        let coordinator = coordinator.clone();
        let cancel = self.server.cancel_token.clone();
        crate::utils::spawn(async move {
            if let Err(e) = coordinator.run().await {
                tracing::error!(error = %e, "trunk coordinator failed");
            }
        });
    } else {
        // Single-node mode: register all trunks directly (existing behavior)
        // ... existing code unchanged ...
    }
    Ok(())
}
```

This keeps full backward compatibility. When no `[cluster]` section is
configured, the existing single-node behavior is preserved unchanged.

### 7.5 Configuration

```toml
[cluster]
enabled = true
node_id = "node1"

[cluster.trunk_coordination]
lease_ttl_secs = 60           # Lease expiry (default: 60)
renewal_interval_secs = 20    # How often to renew (default: 20)
scan_interval_secs = 15       # How often to scan for claimable trunks (default: 15)
max_failure_count = 5          # Release trunk after N consecutive failures (default: 5)
```

---

## 8. Edge Cases and Failure Modes

### 8.1 Database Unavailable

If the coordinator cannot reach PostgreSQL:
- Owned trunks continue their `register_loop` (in-progress registrations
  are not interrupted).
- Lease renewal fails silently; the coordinator retries on the next scan.
- If the outage exceeds the lease TTL, another node may claim the trunk
  once the database recovers. The original owner detects the lost lease
  on its next successful renewal attempt and stops its `register_loop`.

### 8.2 Clock Skew Between Nodes

Lease expiry depends on `CURRENT_TIMESTAMP` in PostgreSQL, not on the
application server's clock. All lease comparisons are server-side SQL.
Clock skew between RustPBX nodes does not affect correctness because the
database is the single source of truth for time comparisons.

### 8.3 Two Nodes Briefly Registering the Same Trunk

During a lease handoff (old lease expired, new node claiming), there is a
brief window where:
1. The old node's `register_loop` may still be running (cancel token not
   yet triggered).
2. The new node sends its first REGISTER.

This results in at most one or two overlapping REGISTER messages. The
provider accepts the latest Contact, so the new node's registration takes
precedence within seconds. This is harmless and self-correcting.

### 8.4 Provider Returns Different Expiry Than Requested

The `register_loop` already handles this: it reads the actual expiry from
the 200 OK response (`registration.expires()`) and schedules re-register
at 85% of that value. The coordinator updates `register_expires` in the
lease table to reflect the provider-granted value.

### 8.5 All Nodes Fail Simultaneously

If all nodes crash, all leases expire. On recovery, each node computes
its preferred assignment and claims its trunks. Since the provider's
registration also expires (typically after 3600s), the first node to
recover and REGISTER restores inbound call delivery.

---

## 9. Migration Path

### 9.1 SeaORM Migration

Add to `src/models/migration.rs`:

```rust
Box::new(super::trunk_lease::Migration),
```

The migration creates the `rustpbx_trunk_registration_leases` table. It is
safe to run on single-node deployments -- the table simply remains empty
when clustering is not enabled.

### 9.2 Rollout Steps

1. Deploy new binary with trunk coordination code to all nodes.
2. Run database migration (creates the lease table).
3. Enable clustering on one node at a time:
   - Add `[cluster]` section to the node's config.
   - Restart the node.
   - Verify it claims its assigned trunks via logs and `/health/detail`.
4. Enable clustering on the remaining nodes.
5. Verify trunk distribution via the health endpoint.

### 9.3 Rollback

Disable clustering by removing the `[cluster]` section from config and
restarting. The node reverts to single-node behavior (registers all
trunks directly). The lease table can be left in place or dropped.
