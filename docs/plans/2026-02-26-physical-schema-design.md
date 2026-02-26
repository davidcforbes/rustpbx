# 4iiz Physical Schema Design

> **Status:** Approved
> **Date:** 2026-02-26
> **Prerequisite:** [Data Storage Architecture Design](2026-02-25-data-storage-architecture-design.md)
> **Next Step:** Implementation plan (DDL scripts, migration runner, SeaORM entity generation)

---

## 1. Purpose

Define the physical schema conventions — primary keys, namespacing, multi-tenancy enforcement, migration tooling, partitioning mechanics, indexing, timestamps, soft deletion, and ORM integration — for all ~80 4iiz tables. These decisions apply uniformly across all six data categories defined in the storage architecture.

---

## 2. Decisions

### 2.1 Schema Independence

The 4iiz schema is designed independently of the existing RustPBX tables. No foreign keys cross the boundary. Migration and data bridging between the two schemas is deferred to a later phase.

Rationale: The existing RustPBX schema (14 tables, `rustpbx_` prefix, i64 PKs, no multi-tenancy) has fundamentally different conventions. A clean break avoids compromise designs that serve neither system well.

### 2.2 Primary Keys: UUID v7

All tables use UUID v7 primary keys.

```sql
id UUID NOT NULL DEFAULT gen_random_uuid(),
```

> Note: `gen_random_uuid()` generates v4. The application layer generates UUID v7 values and supplies them on INSERT. The DEFAULT is a fallback only. The Rust `uuid` crate with `v7` feature provides generation.

**Properties:**
- **Time-ordered** — the first 48 bits encode a millisecond Unix timestamp, so UUIDs sort chronologically by creation time
- **B-tree friendly** — monotonically increasing within a millisecond; no page splits or index fragmentation (unlike UUID v4)
- **Partition-aligned** — on event log tables partitioned by month, UUID v7 values for a given month cluster in the same partition's PK index
- **16 bytes** — stored natively as PostgreSQL `uuid` type
- **Collision-safe** — 74 bits of randomness per millisecond; safe for multi-instance concurrent generation

SeaORM maps `uuid` columns to `Uuid` in Rust via the `uuid` crate.

### 2.3 Namespace: PostgreSQL Schema `iiz`

All 4iiz tables reside in a dedicated PostgreSQL schema:

```sql
CREATE SCHEMA IF NOT EXISTS iiz;
```

Tables use clean, unprefixed names: `iiz.call_records`, `iiz.users`, `iiz.queues`.

The existing RustPBX tables remain in `public` (`public.rustpbx_call_records`, etc.).

Connection-level search path:

```sql
SET search_path = iiz, public;
```

This allows SeaORM entities to reference table names without the schema prefix in application code while maintaining isolation at the database level.

**Benefits:**
- Clean table names (no `iiz_` prefix noise)
- No collision with existing `rustpbx_` tables
- Independent `pg_dump` of either schema
- PostgreSQL schemas are first-class, zero-overhead namespacing

### 2.4 Multi-Tenancy: Row-Level Security (RLS)

Every data table includes an `account_id UUID NOT NULL` column. Tenant isolation is enforced at two layers:

**Layer 1 — Application (explicit):**
Every query includes `WHERE account_id = ?` for clarity and index usage.

**Layer 2 — PostgreSQL RLS (safety net):**

```sql
ALTER TABLE iiz.call_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE iiz.call_records FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON iiz.call_records
    USING (account_id = current_setting('app.current_account_id')::uuid);
```

Each connection from the pool sets the tenant context:

```sql
SET app.current_account_id = '<uuid>';
```

**RLS bypass for admin/system operations:**
- Superuser connections bypass RLS by default
- A dedicated `iiz_admin` role with `BYPASSRLS` handles cross-tenant operations (reporting aggregation, migrations, maintenance)

**Defense in depth:** If application code omits the `account_id` filter, RLS blocks the query rather than returning another tenant's data. This makes cross-tenant data leaks structurally impossible at the database level.

### 2.5 Migration Tooling: Raw SQL + Lightweight Runner

Schema changes are managed as plain SQL files executed by a minimal Rust migration runner.

**File structure:**
```
migrations/
  000_create_schema.sql
  001_create_accounts.sql
  002_create_users.sql
  003_create_call_records.sql
  ...
  000_create_schema_down.sql
  001_create_accounts_down.sql
  ...
```

**Migrations table:**
```sql
CREATE TABLE IF NOT EXISTS iiz.schema_migrations (
    version TEXT PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**Runner behavior:**
1. Read all `*.sql` files from `migrations/` (excluding `*_down.sql`)
2. Query `iiz.schema_migrations` for already-applied versions
3. Execute unapplied migrations in lexicographic order, each in its own transaction
4. Record the version in `schema_migrations` on success
5. On failure, the transaction rolls back and the runner stops

**Rationale:** Over half the DDL is PostgreSQL-specific (partitioning, RLS policies, UNLOGGED tables, PL/pgSQL functions, pgvector indexes). Raw SQL is the natural language for this work. SeaORM remains the runtime ORM for all queries — just not for schema management.

**Rollback:** Each migration has a companion `_down.sql` file. Rollback is manual and deliberate (not automated).

### 2.6 Partitioning: Auto-Creating Monthly Partitions

All 8 high-volume event log tables are range-partitioned by month on their primary timestamp column.

**Parent table definition:**
```sql
CREATE TABLE iiz.call_records (
    id UUID NOT NULL,
    account_id UUID NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    -- ... columns ...
    PRIMARY KEY (id, started_at)
) PARTITION BY RANGE (started_at);
```

> Note: The partition key (`started_at`) must be part of the primary key in partitioned tables. The composite PK `(id, started_at)` is a PostgreSQL requirement, not a design choice.

**Auto-creation function:**

```sql
CREATE OR REPLACE FUNCTION iiz.create_partition_if_not_exists()
RETURNS TRIGGER AS $$
DECLARE
    partition_name TEXT;
    start_date DATE;
    end_date DATE;
BEGIN
    start_date := date_trunc('month', NEW.started_at)::date;
    end_date := (start_date + interval '1 month')::date;
    partition_name := TG_TABLE_SCHEMA || '.' || TG_TABLE_NAME
        || '_' || to_char(start_date, 'YYYY_MM');

    IF NOT EXISTS (
        SELECT 1 FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = TG_TABLE_SCHEMA
          AND c.relname = TG_TABLE_NAME || '_' || to_char(start_date, 'YYYY_MM')
    ) THEN
        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS %s PARTITION OF %I.%I
             FOR VALUES FROM (%L) TO (%L)',
            partition_name, TG_TABLE_SCHEMA, TG_TABLE_NAME,
            start_date, end_date
        );
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

**Belt and suspenders:** On application startup, the migration runner pre-creates partitions for the current and next month. The trigger handles edge cases (late-arriving data, clock skew, month boundaries).

**Partitioned tables:**

| Table | Partition Key |
|-------|--------------|
| call_records | started_at |
| call_flow_events | occurred_at |
| call_transcription_segments | created_at |
| call_ai_summaries | generated_at |
| text_messages | sent_at |
| agent_state_log | changed_at |
| webhook_deliveries | delivered_at |
| api_log_entries | timestamp |

### 2.7 Index Strategy: Index All Filterable Columns

Every column that the UI filters, sorts, or joins on gets an index at table creation time. Monthly partitions keep each index small, so write amplification is bounded.

**Standard indexes per event log partition:**

```sql
-- call_records (per partition, inherited from parent definition)
CREATE INDEX ON iiz.call_records (account_id, started_at);
CREATE INDEX ON iiz.call_records (caller_phone);
CREATE INDEX ON iiz.call_records (callee_phone);
CREATE INDEX ON iiz.call_records (source_id);
CREATE INDEX ON iiz.call_records (agent_id);
CREATE INDEX ON iiz.call_records (status);
CREATE INDEX ON iiz.call_records (queue_id);
```

**Standard indexes for config/compliance tables:**

```sql
-- Every config table gets at minimum:
CREATE INDEX ON iiz.<table> (account_id);
-- Plus indexes on columns used in lookups/joins (FK targets, name, status, etc.)
```

**Soft-delete partial indexes:**

```sql
-- On tables with high read volume, a partial index excludes deleted rows:
CREATE INDEX ON iiz.<table> (account_id)
    WHERE deleted_at IS NULL;
```

**Rationale:** Starting lean and adding indexes later is sound theory, but in practice "later" often means "after users report slow queries." With monthly partitions, each index covers at most one month of data — the write cost is small and the operational simplicity of "every filter has an index" is worth it.

### 2.8 Timestamps: `TIMESTAMPTZ` + UTC

All timestamp columns use `TIMESTAMPTZ` (timestamp with time zone). All values are stored in UTC.

```sql
created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
deleted_at TIMESTAMPTZ,
```

**Connection-level enforcement:**

```sql
SET timezone = 'UTC';
```

**Rust mapping:** `chrono::DateTime<Utc>` via SeaORM. Display-layer converts to the user's configured timezone.

**Why TIMESTAMPTZ, not TIMESTAMP:**
- PostgreSQL internally stores both as 8-byte UTC microseconds, but `TIMESTAMP` (without timezone) is interpreted as "local time" — if the server timezone ever changes, all values shift
- `TIMESTAMPTZ` is unambiguous: always UTC in, timezone-converted on display
- DST transitions, cross-timezone queries, and duration math are all correct by default

### 2.9 Soft Delete: `deleted_at` on Every Table

Every table includes a nullable `deleted_at TIMESTAMPTZ` column. Deletion is a two-phase process:

**Phase 1 — Mark:** Application sets `deleted_at = now()`. RLS policies and application queries filter `WHERE deleted_at IS NULL`, making the row invisible to normal operations.

**Phase 2 — Purge:** A maintenance job physically removes rows where `deleted_at` is older than the retention window. This runs during low-traffic periods when table-level locks are acceptable.

```sql
-- Column on every table:
deleted_at TIMESTAMPTZ,

-- Default RLS policy includes soft-delete filter:
CREATE POLICY tenant_isolation ON iiz.<table>
    USING (
        account_id = current_setting('app.current_account_id')::uuid
        AND deleted_at IS NULL
    );
```

**Uniform pattern:** No table-by-table decisions about whether soft delete applies. Every table supports it. The purge job is configurable per table (e.g., compliance tables retain 7 years, ephemeral tables purge after 24 hours).

**Partial indexes** on high-read tables ensure that `WHERE deleted_at IS NULL` queries use an index that excludes deleted rows entirely — zero scan overhead.

### 2.10 Entity Generation: Schema-First

SeaORM entity structs are generated from the live database schema, not hand-written.

**Workflow:**
1. Write and apply SQL migration
2. Run `sea-orm-cli generate entity --database-url <url> --database-schema iiz --output-dir src/entities/generated/`
3. Generated files are checked into git (they are the canonical Rust representation of the schema)
4. Custom impl blocks, derived traits, and business logic go in separate files (e.g., `src/entities/call_record_ext.rs`) that import the generated entity

**Regeneration:** After any migration, re-run the generator. Diff the output to verify only expected changes.

**Naming:** `sea-orm-cli` converts `snake_case` SQL names to `PascalCase` Rust structs automatically. Table `call_records` becomes `CallRecords` entity.

---

## 3. Common Column Patterns

### 3.1 Base Columns (every table)

```sql
id          UUID        NOT NULL DEFAULT gen_random_uuid(),
account_id  UUID        NOT NULL,
created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
deleted_at  TIMESTAMPTZ,
```

### 3.2 Event Log Tables (append-only, partitioned)

```sql
id          UUID        NOT NULL,  -- UUID v7, application-generated
account_id  UUID        NOT NULL,
-- ... domain columns ...
created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
deleted_at  TIMESTAMPTZ,
-- No updated_at — immutable after INSERT
PRIMARY KEY (id, <partition_key>)
```

> `updated_at` is omitted on event log tables. These rows are never updated (corrections are compensating records). `deleted_at` is retained for the uniform soft-delete pattern, though event log rows are typically purged by dropping old partitions rather than individual soft deletes.

### 3.3 Ephemeral Tables (UNLOGGED)

```sql
CREATE UNLOGGED TABLE iiz.active_calls (
    id          UUID        NOT NULL PRIMARY KEY,
    account_id  UUID        NOT NULL,
    -- ... domain columns ...
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at  TIMESTAMPTZ
);
```

> UNLOGGED tables skip the WAL for ~2x write performance. Data survives normal restarts but is lost on crash — acceptable for state that can be rebuilt from SIP sessions and agent re-registration.

### 3.4 Config / Compliance Tables (standard)

```sql
CREATE TABLE iiz.queues (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL,
    name            TEXT        NOT NULL,
    -- ... domain columns ...
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,
    CONSTRAINT uq_queues_account_name UNIQUE (account_id, name)
);

CREATE INDEX ON iiz.queues (account_id) WHERE deleted_at IS NULL;
```

---

## 4. RLS Policy Template

Applied to every table via migration:

```sql
-- Enable RLS
ALTER TABLE iiz.<table> ENABLE ROW LEVEL SECURITY;
ALTER TABLE iiz.<table> FORCE ROW LEVEL SECURITY;

-- Tenant isolation + soft delete
CREATE POLICY tenant_isolation ON iiz.<table>
    FOR ALL
    USING (
        account_id = current_setting('app.current_account_id')::uuid
        AND deleted_at IS NULL
    )
    WITH CHECK (
        account_id = current_setting('app.current_account_id')::uuid
    );
```

The `USING` clause filters reads (SELECT) and row-targeting writes (UPDATE, DELETE). The `WITH CHECK` clause validates new/modified rows (INSERT, UPDATE). Together they ensure:
- A tenant can only see their own non-deleted rows
- A tenant can only create/modify rows with their own `account_id`
- Deleted rows are invisible without a separate query filter

---

## 5. LISTEN/NOTIFY Trigger Template

Config table writes fire a notification for cache invalidation:

```sql
CREATE OR REPLACE FUNCTION iiz.notify_config_change()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify(
        'iiz_config_changed',
        json_build_object(
            'table', TG_TABLE_NAME,
            'op', TG_OP,
            'id', COALESCE(NEW.id, OLD.id)
        )::text
    );
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Applied to each config table:
CREATE TRIGGER notify_change
    AFTER INSERT OR UPDATE OR DELETE ON iiz.<table>
    FOR EACH ROW EXECUTE FUNCTION iiz.notify_config_change();
```

Application instances listen on the `iiz_config_changed` channel and invalidate/reload the affected moka cache entry.

---

## 6. Maintenance Job: Soft-Delete Purge

A background task physically removes soft-deleted rows past their retention window:

```sql
-- Example: purge config rows deleted more than 90 days ago
DELETE FROM iiz.queues
WHERE deleted_at IS NOT NULL
  AND deleted_at < now() - interval '90 days';
```

**Retention windows (configurable per table category):**

| Category | Retention After Soft Delete |
|----------|:---------------------------:|
| Configuration | 90 days |
| Compliance | 7 years |
| Event Log | Managed by partition drops, not row deletes |
| Ephemeral | 24 hours |
| Analytical | 90 days |

The purge job runs during scheduled maintenance windows. For large tables, it operates in batches with `LIMIT` to avoid long-running transactions.

---

## 7. Summary of Physical Conventions

| Aspect | Convention |
|--------|-----------|
| Primary key | `id UUID NOT NULL` — UUID v7, application-generated |
| Tenant column | `account_id UUID NOT NULL` on every table |
| Timestamps | `TIMESTAMPTZ`, stored UTC, `created_at` + `updated_at` + `deleted_at` |
| Soft delete | `deleted_at TIMESTAMPTZ` on every table; two-phase (mark + purge) |
| Namespace | PostgreSQL schema `iiz`; clean table names |
| Tenant enforcement | RLS policy per table + application-layer `WHERE account_id = ?` |
| Partitioning | Monthly range on timestamp; auto-created via PL/pgSQL trigger |
| Indexes | All filterable columns indexed at creation; partial indexes for `deleted_at IS NULL` |
| Migrations | Raw `.sql` files; lightweight Rust runner; `iiz.schema_migrations` tracking |
| ORM entities | Generated from DB via `sea-orm-cli`; custom logic in separate `_ext.rs` files |
| Cache invalidation | PG trigger → `NOTIFY iiz_config_changed` → moka cache reload |
| Connection timezone | `SET timezone = 'UTC'` on every connection |
| Search path | `SET search_path = iiz, public` on every connection |
