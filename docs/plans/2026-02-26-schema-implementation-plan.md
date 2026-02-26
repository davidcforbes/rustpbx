# 4iiz Schema Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create the complete 4iiz PostgreSQL schema (~80 tables) with a lightweight migration runner, RLS policies, partitioning, and SeaORM entity generation.

**Architecture:** Raw SQL migration files applied by a minimal Rust runner. Tables organized by FK dependency order across 9 domain shards. SeaORM entities generated from the live schema after migrations run.

**Tech Stack:** PostgreSQL 16+, SeaORM 1.1.19, sqlx 0.8.6, uuid (v7), chrono, pgvector

---

## File Structure

```
migrations/
  iiz/
    000_schema_and_functions.sql
    001_enum_types.sql
    010_accounts.sql
    011_users.sql
    012_contacts_and_compliance.sql
    020_tracking_sources.sql
    021_receiving_numbers.sql
    022_number_pools.sql
    023_call_settings.sql
    024_tracking_numbers.sql
    025_text_numbers.sql
    026_target_numbers.sql
    030_schedules.sql
    031_voice_menus.sql
    032_queues.sql
    033_routers.sql
    034_routing_tables.sql
    035_agent_scripts.sql
    036_voicemail.sql
    040_workflows.sql
    041_triggers.sql
    042_lambdas.sql
    043_webhooks.sql
    044_engagement.sql
    045_keyword_spotting.sql
    046_chat_widgets.sql
    050_ai_configs.sql
    051_knowledge_banks.sql
    052_ai_agents.sql
    060_tags_and_scoring.sql
    061_reports_and_notifications.sql
    062_call_daily_summary.sql
    070_compliance_business.sql
    071_compliance_registrations.sql
    072_compliance_requirements.sql
    100_call_records.sql
    101_call_flow_events.sql
    102_call_supplementary.sql
    103_text_records.sql
    104_other_activity_records.sql
    105_agent_state_log.sql
    106_webhook_deliveries.sql
    107_api_log_entries.sql
    108_other_event_logs.sql
    120_ephemeral_tables.sql
    121_knowledge_embeddings.sql
    130_rls_policies.sql
    131_notify_triggers.sql
    132_initial_partitions.sql

    # Rollback files (companion _down.sql for each)
    000_schema_and_functions_down.sql
    ...

src/iiz/
  mod.rs              # Module root, re-exports
  migrator.rs         # SQL migration runner
  pool.rs             # 4-pool connection manager
  entities/
    generated/        # sea-orm-cli output (gitignored initially, committed after generation)
    mod.rs            # Re-exports generated entities
    extensions/       # Custom impl blocks per entity
      mod.rs
      call_record_ext.rs
      ...
```

---

## Task 1: Migration Runner

Create the lightweight SQL migration runner in Rust.

**Files:**
- Create: `src/iiz/mod.rs`
- Create: `src/iiz/migrator.rs`
- Create: `migrations/iiz/.gitkeep`
- Modify: `src/lib.rs` (add `pub mod iiz;`)
- Modify: `Cargo.toml` (add `sqlx` postgres feature, `uuid` v7 feature)
- Test: `src/iiz/migrator.rs` (inline `#[cfg(test)]` module)

**Step 1: Update Cargo.toml dependencies**

```toml
# In [dependencies] section, update existing entries:
uuid = { version = "1.19.0", features = ["v4", "v7"] }

# Ensure sqlx postgres feature is present (it already is):
# sqlx = { version = "0.8.6", features = ["runtime-tokio-rustls", "postgres", "sqlite", "mysql", "any"] }
```

**Step 2: Create module structure**

`src/iiz/mod.rs`:
```rust
pub mod migrator;
```

`src/lib.rs` — add at end:
```rust
pub mod iiz;
```

**Step 3: Write the migration runner**

`src/iiz/migrator.rs`:
```rust
use sqlx::PgPool;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// A single migration file
#[derive(Debug, Clone)]
pub struct Migration {
    pub version: String,
    pub name: String,
    pub sql: String,
}

/// Lightweight SQL migration runner for the iiz schema.
///
/// Reads .sql files from a directory, tracks applied versions
/// in `iiz.schema_migrations`, and applies new ones in order.
pub struct Migrator {
    migrations_dir: PathBuf,
}

impl Migrator {
    pub fn new(migrations_dir: impl Into<PathBuf>) -> Self {
        Self {
            migrations_dir: migrations_dir.into(),
        }
    }

    /// Load all forward migration files (exclude *_down.sql) sorted by filename.
    pub fn load_migrations(dir: &Path) -> Result<Vec<Migration>, Box<dyn std::error::Error>> {
        let mut migrations = Vec::new();
        let mut entries: Vec<_> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.ends_with(".sql") && !name.ends_with("_down.sql")
            })
            .collect();

        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let filename = entry.file_name().to_string_lossy().to_string();
            let version = filename.trim_end_matches(".sql").to_string();
            let sql = std::fs::read_to_string(entry.path())?;
            migrations.push(Migration {
                version: version.clone(),
                name: filename,
                sql,
            });
        }

        Ok(migrations)
    }

    /// Ensure the schema and migrations tracking table exist.
    async fn ensure_tracking_table(pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE SCHEMA IF NOT EXISTS iiz;
            CREATE TABLE IF NOT EXISTS iiz.schema_migrations (
                version TEXT PRIMARY KEY,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
            );
            "#,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Get list of already-applied migration versions.
    async fn applied_versions(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT version FROM iiz.schema_migrations ORDER BY version")
                .fetch_all(pool)
                .await?;
        Ok(rows.into_iter().map(|(v,)| v).collect())
    }

    /// Run all pending migrations. Returns count of newly applied migrations.
    pub async fn run(&self, pool: &PgPool) -> Result<usize, Box<dyn std::error::Error>> {
        Self::ensure_tracking_table(pool).await?;

        let all_migrations = Self::load_migrations(&self.migrations_dir)?;
        let applied = Self::applied_versions(pool).await?;
        let applied_set: std::collections::HashSet<&str> =
            applied.iter().map(|s| s.as_str()).collect();

        let pending: Vec<&Migration> = all_migrations
            .iter()
            .filter(|m| !applied_set.contains(m.version.as_str()))
            .collect();

        if pending.is_empty() {
            info!("iiz schema: all {} migrations already applied", all_migrations.len());
            return Ok(0);
        }

        info!(
            "iiz schema: applying {} pending migration(s) ({} total)",
            pending.len(),
            all_migrations.len()
        );

        for migration in &pending {
            info!("  applying: {}", migration.name);

            // Each migration runs in its own transaction
            let mut tx = pool.begin().await?;

            // Split on semicolons and execute each statement
            // (PostgreSQL doesn't support multi-statement in a single query via sqlx)
            for statement in split_sql_statements(&migration.sql) {
                let trimmed = statement.trim();
                if trimmed.is_empty() || trimmed.starts_with("--") {
                    continue;
                }
                sqlx::query(trimmed).execute(&mut *tx).await.map_err(|e| {
                    format!(
                        "Migration {} failed on statement:\n{}\nError: {}",
                        migration.name,
                        truncate(trimmed, 200),
                        e
                    )
                })?;
            }

            // Record the migration
            sqlx::query("INSERT INTO iiz.schema_migrations (version) VALUES ($1)")
                .bind(&migration.version)
                .execute(&mut *tx)
                .await?;

            tx.commit().await?;
            info!("  applied:  {}", migration.name);
        }

        Ok(pending.len())
    }

    /// Roll back a specific migration version using its _down.sql companion.
    pub async fn rollback(
        &self,
        pool: &PgPool,
        version: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let down_file = self.migrations_dir.join(format!("{}_down.sql", version));
        if !down_file.exists() {
            return Err(format!("No rollback file found: {}", down_file.display()).into());
        }

        let sql = std::fs::read_to_string(&down_file)?;
        let mut tx = pool.begin().await?;

        for statement in split_sql_statements(&sql) {
            let trimmed = statement.trim();
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }
            sqlx::query(trimmed).execute(&mut *tx).await?;
        }

        sqlx::query("DELETE FROM iiz.schema_migrations WHERE version = $1")
            .bind(version)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        warn!("Rolled back migration: {}", version);
        Ok(())
    }
}

/// Split SQL text into individual statements on semicolons,
/// respecting dollar-quoted strings ($$...$$) and single-quoted strings.
fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut chars = sql.chars().peekable();
    let mut in_single_quote = false;
    let mut in_dollar_quote = false;
    let mut dollar_tag = String::new();

    while let Some(ch) = chars.next() {
        current.push(ch);

        if in_single_quote {
            if ch == '\'' {
                // Check for escaped quote ''
                if chars.peek() == Some(&'\'') {
                    current.push(chars.next().unwrap());
                } else {
                    in_single_quote = false;
                }
            }
            continue;
        }

        if in_dollar_quote {
            // Check if we've hit the closing dollar tag
            if ch == '$' {
                let mut tag = String::from("$");
                while let Some(&next) = chars.peek() {
                    tag.push(chars.next().unwrap());
                    current.push(tag.chars().last().unwrap());
                    if next == '$' {
                        break;
                    }
                }
                if tag == dollar_tag {
                    in_dollar_quote = false;
                    dollar_tag.clear();
                }
            }
            continue;
        }

        match ch {
            '\'' => {
                in_single_quote = true;
            }
            '$' => {
                // Detect dollar quoting: $$ or $tag$
                let mut tag = String::from("$");
                let saved_pos: Vec<char> = Vec::new();
                let mut found_dollar_quote = false;

                // Peek ahead for the closing $
                let mut peek_chars: Vec<char> = Vec::new();
                while let Some(&next) = chars.peek() {
                    peek_chars.push(chars.next().unwrap());
                    tag.push(*peek_chars.last().unwrap());
                    if *peek_chars.last().unwrap() == '$' {
                        found_dollar_quote = true;
                        break;
                    }
                    // Only allow alphanumeric and underscore in tag
                    if !peek_chars.last().unwrap().is_alphanumeric()
                        && *peek_chars.last().unwrap() != '_'
                    {
                        break;
                    }
                }

                for c in &peek_chars {
                    current.push(*c);
                }
                drop(saved_pos);

                if found_dollar_quote {
                    in_dollar_quote = true;
                    dollar_tag = tag;
                }
            }
            ';' => {
                if !in_single_quote && !in_dollar_quote {
                    let stmt = current.trim().to_string();
                    if !stmt.is_empty() && stmt != ";" {
                        statements.push(stmt);
                    }
                    current.clear();
                }
            }
            '-' => {
                // Skip line comments
                if chars.peek() == Some(&'-') {
                    current.push(chars.next().unwrap());
                    while let Some(&next) = chars.peek() {
                        if next == '\n' {
                            break;
                        }
                        current.push(chars.next().unwrap());
                    }
                }
            }
            _ => {}
        }
    }

    // Don't forget trailing statement without semicolon
    let remaining = current.trim().to_string();
    if !remaining.is_empty() && remaining != ";" && !remaining.starts_with("--") {
        statements.push(remaining);
    }

    statements
}

fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        &s[..max_len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_simple_statements() {
        let sql = "CREATE TABLE a (id INT); CREATE TABLE b (id INT);";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 2);
        assert!(stmts[0].contains("CREATE TABLE a"));
        assert!(stmts[1].contains("CREATE TABLE b"));
    }

    #[test]
    fn test_split_preserves_dollar_quoted_functions() {
        let sql = r#"
CREATE FUNCTION foo() RETURNS void AS $$
BEGIN
    RAISE NOTICE 'hello; world';
END;
$$ LANGUAGE plpgsql;

CREATE TABLE bar (id INT);
"#;
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 2, "Got: {:?}", stmts);
        assert!(stmts[0].contains("RAISE NOTICE"));
        assert!(stmts[1].contains("CREATE TABLE bar"));
    }

    #[test]
    fn test_split_preserves_single_quoted_strings() {
        let sql = "INSERT INTO t (v) VALUES ('hello; world'); SELECT 1;";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 2);
        assert!(stmts[0].contains("hello; world"));
    }

    #[test]
    fn test_split_skips_comments() {
        let sql = "-- This is a comment\nCREATE TABLE t (id INT);";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 1);
    }

    #[test]
    fn test_load_migrations_sorts_by_filename() {
        let dir = std::env::temp_dir().join("test_migrations_sort");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(dir.join("002_second.sql"), "SELECT 2;").unwrap();
        std::fs::write(dir.join("001_first.sql"), "SELECT 1;").unwrap();
        std::fs::write(dir.join("001_first_down.sql"), "DROP;").unwrap(); // should be excluded

        let migrations = Migrator::load_migrations(&dir).unwrap();
        assert_eq!(migrations.len(), 2);
        assert_eq!(migrations[0].version, "001_first");
        assert_eq!(migrations[1].version, "002_second");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
```

**Step 4: Run tests**

```bash
cargo test -p rustpbx iiz::migrator::tests -- --nocapture
```

Expected: All 4 tests pass.

**Step 5: Commit**

```bash
git add src/iiz/ src/lib.rs Cargo.toml migrations/
git commit -m "feat(iiz): add lightweight SQL migration runner"
```

---

## Task 2: Foundation Migration — Schema, Functions, Enum Types

**Files:**
- Create: `migrations/iiz/000_schema_and_functions.sql`
- Create: `migrations/iiz/000_schema_and_functions_down.sql`
- Create: `migrations/iiz/001_enum_types.sql`
- Create: `migrations/iiz/001_enum_types_down.sql`

**Step 1: Write schema and infrastructure functions**

`migrations/iiz/000_schema_and_functions.sql`:
```sql
-- 4iiz Schema Foundation
-- Creates the iiz schema, migration tracking, and shared infrastructure functions.

CREATE SCHEMA IF NOT EXISTS iiz;

-- Migration tracking (idempotent — migrator also creates this)
CREATE TABLE IF NOT EXISTS iiz.schema_migrations (
    version TEXT PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- updated_at auto-trigger function
CREATE OR REPLACE FUNCTION iiz.set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Partition auto-creation function.
-- Called by a BEFORE INSERT trigger on partitioned tables.
-- Dynamically creates monthly partitions if they don't exist.
CREATE OR REPLACE FUNCTION iiz.create_monthly_partition()
RETURNS TRIGGER AS $$
DECLARE
    partition_name TEXT;
    start_date DATE;
    end_date DATE;
    partition_col TEXT;
BEGIN
    -- The partition column name is passed via TG_ARGV[0]
    partition_col := TG_ARGV[0];

    -- Extract the month boundary from the NEW row's partition column
    EXECUTE format('SELECT date_trunc(''month'', ($1).%I)::date', partition_col)
        INTO start_date USING NEW;
    end_date := (start_date + interval '1 month')::date;

    partition_name := format('%I.%I', TG_TABLE_SCHEMA,
        TG_TABLE_NAME || '_' || to_char(start_date, 'YYYY_MM'));

    -- Create partition if it doesn't exist
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
        RAISE NOTICE 'Created partition: %', partition_name;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Config change notification function for cache invalidation.
-- Applied as AFTER trigger on config tables.
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

-- Helper: apply updated_at trigger to a table
CREATE OR REPLACE FUNCTION iiz.add_updated_at_trigger(table_name TEXT)
RETURNS void AS $$
BEGIN
    EXECUTE format(
        'CREATE TRIGGER set_updated_at
         BEFORE UPDATE ON iiz.%I
         FOR EACH ROW EXECUTE FUNCTION iiz.set_updated_at()',
        table_name
    );
END;
$$ LANGUAGE plpgsql;

-- Helper: apply config change notification trigger to a table
CREATE OR REPLACE FUNCTION iiz.add_notify_trigger(table_name TEXT)
RETURNS void AS $$
BEGIN
    EXECUTE format(
        'CREATE TRIGGER notify_change
         AFTER INSERT OR UPDATE OR DELETE ON iiz.%I
         FOR EACH ROW EXECUTE FUNCTION iiz.notify_config_change()',
        table_name
    );
END;
$$ LANGUAGE plpgsql;
```

`migrations/iiz/000_schema_and_functions_down.sql`:
```sql
DROP FUNCTION IF EXISTS iiz.add_notify_trigger(TEXT);
DROP FUNCTION IF EXISTS iiz.add_updated_at_trigger(TEXT);
DROP FUNCTION IF EXISTS iiz.notify_config_change();
DROP FUNCTION IF EXISTS iiz.create_monthly_partition();
DROP FUNCTION IF EXISTS iiz.set_updated_at();
-- Do NOT drop iiz.schema_migrations or the schema itself in rollback
```

**Step 2: Write enum types**

`migrations/iiz/001_enum_types.sql`:
```sql
-- Shared enum types used across multiple tables.
-- PostgreSQL enums are more storage-efficient and type-safe than TEXT CHECK constraints.
-- Only create enums for values that are truly fixed and used in multiple tables.
-- Single-table or frequently-changing enums use TEXT with CHECK constraints instead.

-- Call direction (used by call_records, text_records, fax_records, active_calls)
CREATE TYPE iiz.call_direction AS ENUM ('inbound', 'outbound', 'internal');

-- Call status (used by call_records)
CREATE TYPE iiz.call_status AS ENUM ('answered', 'missed', 'voicemail', 'in_progress', 'failed');

-- Communication channel (used by chat_records, notifications)
CREATE TYPE iiz.channel_type AS ENUM ('web_chat', 'sms', 'whatsapp');

-- User role (used by users)
CREATE TYPE iiz.user_role AS ENUM ('admin', 'agent', 'supervisor');

-- Agent presence status (used by presence, agent_state_log)
CREATE TYPE iiz.agent_status AS ENUM ('available', 'on_call', 'after_call_work', 'offline', 'break', 'dnd');

-- Queue strategy (used by queues)
CREATE TYPE iiz.queue_strategy AS ENUM ('ring_all', 'round_robin', 'longest_idle', 'weighted');

-- Routing destination type (polymorphic, used across routing entities)
-- Stored as TEXT on tables (not enum) because new destination types may be added.

-- Monitor mode (used by monitoring_events, active_calls)
CREATE TYPE iiz.monitor_mode AS ENUM ('listen', 'whisper', 'barge');

-- Export format (used by export_records)
CREATE TYPE iiz.export_format AS ENUM ('csv', 'pdf', 'excel');

-- Transcription speaker (used by call_transcription_segments)
CREATE TYPE iiz.speaker_type AS ENUM ('agent', 'caller', 'system');

-- Number class (used by tracking_numbers)
CREATE TYPE iiz.number_class AS ENUM ('local', 'toll_free');

-- Number type (used by tracking_numbers)
CREATE TYPE iiz.number_type AS ENUM ('offsite_static', 'onsite_dynamic');

-- Workflow node type
CREATE TYPE iiz.workflow_node_type AS ENUM ('event', 'condition', 'action');

-- SIP transport (used by locations)
CREATE TYPE iiz.sip_transport AS ENUM ('udp', 'tcp', 'tls', 'wss');

-- Active call status (used by active_calls)
CREATE TYPE iiz.active_call_status AS ENUM ('ringing', 'active', 'on_hold', 'transferring', 'wrapping');

-- Account type
CREATE TYPE iiz.account_type AS ENUM ('agency', 'standard');

-- Account status
CREATE TYPE iiz.account_status AS ENUM ('active', 'suspended', 'closed');

-- Summary type (used by call_ai_summaries)
CREATE TYPE iiz.summary_type AS ENUM (
    'classic', 'customer_success', 'key_insights', 'action_items',
    'sentiment_analysis', 'lead_qualification', 'compliance_review',
    'topic_classification', 'custom'
);

-- Greeting type (used by voice_menus, voicemail_boxes)
CREATE TYPE iiz.greeting_type AS ENUM ('audio', 'tts', 'default', 'none');

-- Compliance status (used across trust center entities)
CREATE TYPE iiz.compliance_status AS ENUM ('draft', 'submitted', 'pending', 'in_progress', 'approved', 'rejected', 'suspended', 'expired', 'completed', 'not_started', 'not_applicable', 'not_registered');

-- STIR/SHAKEN attestation level
CREATE TYPE iiz.attestation_level AS ENUM ('a', 'b', 'c');
```

`migrations/iiz/001_enum_types_down.sql`:
```sql
-- Drop all enum types in reverse order
DROP TYPE IF EXISTS iiz.attestation_level;
DROP TYPE IF EXISTS iiz.compliance_status;
DROP TYPE IF EXISTS iiz.greeting_type;
DROP TYPE IF EXISTS iiz.summary_type;
DROP TYPE IF EXISTS iiz.account_status;
DROP TYPE IF EXISTS iiz.account_type;
DROP TYPE IF EXISTS iiz.active_call_status;
DROP TYPE IF EXISTS iiz.sip_transport;
DROP TYPE IF EXISTS iiz.workflow_node_type;
DROP TYPE IF EXISTS iiz.number_type;
DROP TYPE IF EXISTS iiz.number_class;
DROP TYPE IF EXISTS iiz.speaker_type;
DROP TYPE IF EXISTS iiz.export_format;
DROP TYPE IF EXISTS iiz.monitor_mode;
DROP TYPE IF EXISTS iiz.queue_strategy;
DROP TYPE IF EXISTS iiz.agent_status;
DROP TYPE IF EXISTS iiz.user_role;
DROP TYPE IF EXISTS iiz.channel_type;
DROP TYPE IF EXISTS iiz.call_status;
DROP TYPE IF EXISTS iiz.call_direction;
```

**Step 3: Commit**

```bash
git add migrations/iiz/
git commit -m "feat(iiz): add foundation migrations — schema, functions, enum types"
```

---

## Task 3: Core Entity Migrations — Accounts & Users

These are FK targets for nearly every other table. Must be created first.

**Files:**
- Create: `migrations/iiz/010_accounts.sql`
- Create: `migrations/iiz/011_users.sql`
- Create: `migrations/iiz/012_contacts_and_compliance.sql`

**Step 1: Write accounts migration**

`migrations/iiz/010_accounts.sql`:
```sql
-- Accounts: multi-tenant root entity.
-- Agency accounts can own Standard sub-accounts (max depth = 2).

CREATE TABLE iiz.accounts (
    id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    name                TEXT        NOT NULL,
    account_type        iiz.account_type NOT NULL DEFAULT 'standard',
    parent_account_id   UUID        REFERENCES iiz.accounts(id) ON DELETE SET NULL,
    slug                TEXT        NOT NULL,
    timezone            TEXT        NOT NULL DEFAULT 'America/New_York',
    status              iiz.account_status NOT NULL DEFAULT 'active',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ,

    CONSTRAINT uq_accounts_slug UNIQUE (slug)
);

CREATE INDEX idx_accounts_parent ON iiz.accounts (parent_account_id) WHERE parent_account_id IS NOT NULL;
CREATE INDEX idx_accounts_status ON iiz.accounts (status) WHERE deleted_at IS NULL;

SELECT iiz.add_updated_at_trigger('accounts');
SELECT iiz.add_notify_trigger('accounts');
```

**Step 2: Write users migration**

`migrations/iiz/011_users.sql`:
```sql
-- Users: authentication, authorization, and agent identity.

CREATE TABLE iiz.users (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    username        TEXT        NOT NULL,
    email           TEXT        NOT NULL,
    password_hash   TEXT        NOT NULL,
    display_name    TEXT,
    initials        TEXT,
    avatar_color    TEXT        DEFAULT '#00bcd4',
    role            iiz.user_role NOT NULL DEFAULT 'agent',
    phone           TEXT,
    is_active       BOOLEAN     NOT NULL DEFAULT true,
    reset_token     TEXT,
    reset_token_expires TIMESTAMPTZ,
    last_login_at   TIMESTAMPTZ,
    last_login_ip   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,

    CONSTRAINT uq_users_account_username UNIQUE (account_id, username),
    CONSTRAINT uq_users_email UNIQUE (email)
);

CREATE INDEX idx_users_account ON iiz.users (account_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_role ON iiz.users (account_id, role) WHERE deleted_at IS NULL;

SELECT iiz.add_updated_at_trigger('users');
SELECT iiz.add_notify_trigger('users');
```

**Step 3: Write contacts and compliance list migration**

`migrations/iiz/012_contacts_and_compliance.sql`:
```sql
-- Contact lists, blocked numbers, DNC/DNT entries.
-- Grouped together because they're simple, low-FK-dependency entities.

-- Contact Lists
CREATE TABLE iiz.contact_lists (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    description     TEXT,
    member_count    INTEGER     NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,

    CONSTRAINT uq_contact_lists_account_name UNIQUE (account_id, name)
);

CREATE INDEX idx_contact_lists_account ON iiz.contact_lists (account_id) WHERE deleted_at IS NULL;

SELECT iiz.add_updated_at_trigger('contact_lists');
SELECT iiz.add_notify_trigger('contact_lists');

-- Contact List Members
CREATE TABLE iiz.contact_list_members (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    list_id         UUID        NOT NULL REFERENCES iiz.contact_lists(id) ON DELETE CASCADE,
    phone           TEXT        NOT NULL,
    contact_name    TEXT,
    added_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,

    CONSTRAINT uq_contact_list_members_phone UNIQUE (list_id, phone)
);

CREATE INDEX idx_clm_list ON iiz.contact_list_members (list_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_clm_account ON iiz.contact_list_members (account_id) WHERE deleted_at IS NULL;

SELECT iiz.add_updated_at_trigger('contact_list_members');

-- Blocked Numbers
CREATE TABLE iiz.blocked_numbers (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    number          TEXT        NOT NULL,
    cnam            TEXT,
    calls_blocked   INTEGER     NOT NULL DEFAULT 0,
    last_blocked_at TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,

    CONSTRAINT uq_blocked_numbers_account UNIQUE (account_id, number)
);

CREATE INDEX idx_blocked_account ON iiz.blocked_numbers (account_id) WHERE deleted_at IS NULL;

SELECT iiz.add_updated_at_trigger('blocked_numbers');

-- DNC (Do Not Call) Entries
CREATE TABLE iiz.dnc_entries (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    number          TEXT        NOT NULL,
    added_by_id     UUID        REFERENCES iiz.users(id) ON DELETE SET NULL,
    reason          TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,

    CONSTRAINT uq_dnc_account UNIQUE (account_id, number)
);

CREATE INDEX idx_dnc_account ON iiz.dnc_entries (account_id) WHERE deleted_at IS NULL;

SELECT iiz.add_updated_at_trigger('dnc_entries');

-- DNT (Do Not Text) Entries
CREATE TABLE iiz.dnt_entries (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    number          TEXT        NOT NULL,
    e164            TEXT        NOT NULL,
    rejected_count  INTEGER     NOT NULL DEFAULT 0,
    last_rejected_at TIMESTAMPTZ,
    added_by_id     UUID        REFERENCES iiz.users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,

    CONSTRAINT uq_dnt_account UNIQUE (account_id, e164)
);

CREATE INDEX idx_dnt_account ON iiz.dnt_entries (account_id) WHERE deleted_at IS NULL;

SELECT iiz.add_updated_at_trigger('dnt_entries');
```

**Step 4: Commit**

```bash
git add migrations/iiz/
git commit -m "feat(iiz): add core entity migrations — accounts, users, contacts, compliance lists"
```

---

## Task 4: Number Management Migrations (Shard 03)

**Files:**
- Create: `migrations/iiz/020_tracking_sources.sql`
- Create: `migrations/iiz/021_receiving_numbers.sql`
- Create: `migrations/iiz/022_number_pools.sql`
- Create: `migrations/iiz/023_call_settings.sql`
- Create: `migrations/iiz/024_tracking_numbers.sql`
- Create: `migrations/iiz/025_text_numbers.sql`
- Create: `migrations/iiz/026_target_numbers.sql`

**Column reference (from data dictionary shard 03):**

| Table | Columns |
|-------|---------|
| tracking_sources | id, account_id, name, source_type, position, last_touch, number_count, call_count, status (text: Active/Inactive), created/updated/deleted_at |
| receiving_numbers | id, account_id, number, description, tracking_count, total_calls, created/updated/deleted_at |
| number_pools | id, account_id, name, description, source_id (FK tracking_sources), auto_manage, target_accuracy, created/updated/deleted_at |
| number_pool_members | id, account_id, pool_id (FK number_pools), tracking_number_id (UQ), status (text: Active/Inactive), call_count, added_at, created/updated/deleted_at |
| call_settings | id, account_id, name, is_default, greeting_enabled, whisper_enabled, inbound_recording, outbound_recording, transcription_enabled, caller_id_enabled, enhanced_caller_id, caller_id_override, spam_filter_enabled, created/updated/deleted_at |
| tracking_numbers | id, account_id, number (UQ), source_id (FK), routing_description, routing_type (text), routing_target_type, routing_target_id, text_enabled, receiving_number_id (FK), number_type (enum), number_class (enum), pool_id (FK nullable), billing_date, is_active, created/updated/deleted_at |
| text_numbers | id, account_id, number, name, is_assigned, created/updated/deleted_at |
| target_numbers | id, account_id, number, name, description, target_type (text: Phone Match/SIP/Agent), priority, concurrency_cap, weight, status (text: Active/Inactive), created/updated/deleted_at |

**Pattern:** Each file follows the same structure as Task 3 — CREATE TABLE with standard columns (id, account_id, created_at, updated_at, deleted_at), appropriate indexes, `add_updated_at_trigger`, `add_notify_trigger` for config tables.

**Step 1:** Write all 7 migration files following the column specs above. Use TEXT with CHECK constraints for single-table enums. Use FK references where specified. Apply `WHERE deleted_at IS NULL` partial indexes on account_id.

**Step 2: Commit**

```bash
git add migrations/iiz/02*.sql
git commit -m "feat(iiz): add number management migrations — sources, numbers, pools, settings"
```

---

## Task 5: Routing & Call Flow Migrations (Shard 04)

**Files:**
- Create: `migrations/iiz/030_schedules.sql`
- Create: `migrations/iiz/031_voice_menus.sql`
- Create: `migrations/iiz/032_queues.sql`
- Create: `migrations/iiz/033_routers.sql`
- Create: `migrations/iiz/034_routing_tables.sql`
- Create: `migrations/iiz/035_agent_scripts.sql`
- Create: `migrations/iiz/036_voicemail.sql`

**Column reference (from data dictionary shard 04):**

| Table | Key Details |
|-------|-------------|
| schedules | 7 day pairs (mon-sun _open/_close as TIME nullable), timezone, closed_destination_type/id (polymorphic) |
| schedule_holidays | schedule_id FK CASCADE, date (UQ per schedule), name, is_closed, custom_open/close, override_destination_type/id |
| voice_menus | greeting_type (enum), speech_recognition, speech_language, timeout_secs, max_retries, no_input_destination_type/id |
| voice_menu_options | menu_id FK CASCADE, dtmf_digit (UQ per menu), destination_type/id/number |
| queues | strategy (enum), schedule_id FK nullable, repeat_callers, caller_id_display, max_wait_secs, no_answer_destination_type/id, moh_audio_url, wrap_up_secs, is_active |
| queue_agents | queue_id FK CASCADE, agent_id FK, UQ(queue_id, agent_id), priority, is_active |
| smart_routers | name, priority, fallback_destination_type/id, is_active |
| smart_router_rules | router_id FK CASCADE, condition_field, condition_operator, condition_value, destination_type/id/number |
| geo_routers | default_destination_type/id |
| geo_router_rules | router_id FK CASCADE, region, region_type, UQ(router_id, region), destination_type/id/number |
| routing_tables | name, is_active |
| routing_table_routes | table_id FK CASCADE, priority, match_pattern, destination_type/id/number, weight |
| agent_scripts | name, description, content (TEXT with {{variable}} templates) |
| voicemail_boxes | name, max_message_length_secs, greeting_type (enum), greeting_audio_url, transcription_enabled, email_notification_enabled, notification_email, max_messages |
| voicemail_messages | mailbox_id FK CASCADE, call_id FK nullable, caller_number, caller_name, duration_secs, audio_url, transcription, is_read, recorded_at |

**Pattern:** Polymorphic routing uses `destination_type TEXT` + `destination_id UUID` + optional `destination_number TEXT`. No FK constraint on destination_id (resolved in application layer). Child entities CASCADE delete from parents. voicemail_messages is Event Log category (no notify trigger).

**Step 1:** Write all 7 migration files.

**Step 2: Commit**

```bash
git add migrations/iiz/03*.sql
git commit -m "feat(iiz): add routing migrations — schedules, menus, queues, routers, voicemail"
```

---

## Task 6: Automation & Engagement Migrations (Shard 05)

**Files:**
- Create: `migrations/iiz/040_workflows.sql`
- Create: `migrations/iiz/041_triggers.sql`
- Create: `migrations/iiz/042_lambdas.sql`
- Create: `migrations/iiz/043_webhooks.sql`
- Create: `migrations/iiz/044_engagement.sql`
- Create: `migrations/iiz/045_keyword_spotting.sql`
- Create: `migrations/iiz/046_chat_widgets.sql`

**Column reference (from data dictionary shard 05):**

| Table | Key Details |
|-------|-------------|
| workflows | canvas_json (JSONB), status (text: Active/Draft/Paused) |
| workflow_nodes | workflow_id FK CASCADE, node_type (enum), event_type/action_type/condition_type, config_json (JSONB), label, position_x/y |
| workflow_edges | workflow_id FK CASCADE, from_node_id/to_node_id FK, UQ(workflow_id, from_node_id, to_node_id), label, sort_order |
| triggers | trigger_event, run_on, runs_7d (counter), status |
| trigger_conditions | trigger_id FK CASCADE, sort_order, field, operator, value |
| trigger_actions | trigger_id FK CASCADE, sort_order, action_type, action_config (JSONB) |
| lambdas | runtime (text), code (TEXT), handler, timeout_ms, memory_mb, last_invoked_at, invocation_count, error_count |
| lambda_env_vars | lambda_id FK CASCADE, key (UQ per lambda), value (TEXT — encrypted at app layer) |
| webhooks | callback_url, method, body_type, headers (JSONB), secret, retry_count, retry_delay_secs, status, last_triggered_at |
| webhook_subscriptions | webhook_id FK CASCADE, event_type (UQ per webhook) |
| bulk_messages | sender_number_id FK, sender_phone, message_body, msg_type, contact_list_id FK nullable, recipient/sent/delivered/failed counts, status, scheduled/started/completed_at |
| lead_reactor_configs | trigger_event, delay_minutes, is_active, working_hours_only, max_retries |
| lead_reactor_actions | config_id FK CASCADE, sort_order, action_type, template_content, action_config (JSONB) |
| smart_dialer_configs | mode, max_concurrent, ring_timeout_secs, retry_attempts/interval, outbound_number/cnam, start/end_time, timezone, active_days (INTEGER bitmask), contact_list_id FK, agent_script_id FK, is_active |
| form_reactor_entries | form_fields, tracking_number_id FK nullable, call_count, status |
| reminders | timezone, remind_at, is_recurring, recurrence_rule, contact_source, contact_phone, contact_list_id FK, delivery_method, recipient, message, status, call_id FK nullable |
| keyword_spotting_configs | sensitivity (text: Low/Medium/High), apply_to_all_numbers, is_active |
| keyword_spotting_keywords | config_id FK CASCADE, keyword, category (text: Positive/Negative/Neutral), score_weight |
| keyword_spotting_numbers | config_id FK CASCADE, tracking_number_id FK, UQ(config_id, tracking_number_id) |
| chat_widgets | website_url, tracking_number_id FK nullable, routing_type, queue_id FK nullable, agent_count, custom_fields_count, status, config_json (JSONB), chat_count |

**Step 1:** Write all 7 migration files.

**Step 2: Commit**

```bash
git add migrations/iiz/04*.sql
git commit -m "feat(iiz): add automation migrations — workflows, triggers, webhooks, engagement"
```

---

## Task 7: AI Tools Migrations (Shard 06)

**Files:**
- Create: `migrations/iiz/050_ai_configs.sql`
- Create: `migrations/iiz/051_knowledge_banks.sql`
- Create: `migrations/iiz/052_ai_agents.sql`

**Column reference (from data dictionary shard 06):**

| Table | Key Details |
|-------|-------------|
| ask_ai_configs | preset (text enum), custom_prompt, tracking_number_id FK nullable, delay, output_action, workflow_ids (JSONB array), is_active |
| summary_configs | account_id (UQ — singleton), phone/video/chat_enabled, enabled_summary_types (JSONB), transcribe_all, transcription_language, pii_redaction_enabled/rules, default_model |
| knowledge_banks | category (text enum), document_count, total_size_bytes (BIGINT), status (text: Ready/Indexing/Error), last_import_at, used_by |
| knowledge_bank_documents | bank_id FK CASCADE, filename, file_type, source_url, file_ref, content_hash, file_size_bytes (BIGINT), page_count, chunk_count, embedding_status (text), embedding_model, error_message, indexed_at |
| voice_ai_agents | welcome_message, instructions, voice (text enum), language, knowledge_bank_id FK nullable, max_turns, handoff_threshold, handoff_destination_type/id (polymorphic), is_active |
| chat_ai_agents | instructions, knowledge_bank_id FK nullable, welcome_message, max_turns, handoff_threshold, handoff_queue_id FK nullable, is_active |
| chat_ai_configs | knowledge_bank_id FK nullable, instructions, max_turns, handoff_threshold, crm_integration_enabled, crm_type, crm_config (JSONB — encrypted at app layer), is_active |
| dialogflow_configs | project_id, service_account_json (TEXT — encrypted at app layer), language, default_intent, fallback_message, connection_status (text), last_tested_at, is_active |

**Step 1:** Write all 3 migration files.

**Step 2: Commit**

```bash
git add migrations/iiz/05*.sql
git commit -m "feat(iiz): add AI tools migrations — configs, knowledge banks, agents"
```

---

## Task 8: Analytics & Reporting Migrations (Shard 07)

**Files:**
- Create: `migrations/iiz/060_tags_and_scoring.sql`
- Create: `migrations/iiz/061_reports_and_notifications.sql`
- Create: `migrations/iiz/062_call_daily_summary.sql`

**Column reference (from data dictionary shard 07):**

| Table | Key Details |
|-------|-------------|
| tags | name (UQ per account), color (TEXT hex), description, usage_count |
| call_tags | call_id (FK call_records — deferred, created later), tag_id FK tags, UQ(call_id, tag_id), applied_at, applied_by_type, applied_by_id FK users nullable, trigger_id (deferred FK triggers nullable) |
| scoring_configs | account_id (UQ — singleton), answer_rate/talk_time/conversion weights (0-100, sum=100), min_talk_time_secs, target_answer_rate |
| appointments | call_id (deferred FK nullable), scheduled_at, caller_name, caller_phone, source_id (deferred FK nullable), agent_id FK, appointment_type, status, revenue (NUMERIC(12,2) nullable), notes |
| custom_reports | name, report_type, columns (JSONB), filters (JSONB), date_range_type, custom_start/end_date, sort_column, sort_direction, schedule (TEXT cron nullable), schedule_recipients (JSONB), last_run_at, created_by_id FK users, is_shared |
| notification_rules | metric, condition_operator, threshold_value (NUMERIC), time_window_minutes, notification_method, recipients (JSONB), cooldown_minutes, is_active, last_triggered_at, trigger_count |
| call_daily_summary | summary_date (DATE), source_id (UUID nullable), agent_id (UUID nullable), queue_id (UUID nullable), UQ(account_id, summary_date, source_id, agent_id, queue_id), 12+ measure columns (INTEGER/NUMERIC), computed_at |

**Note on deferred FKs:** call_tags and appointments reference call_records which is created in Task 10. Use UUID columns without FK constraints initially; add FK constraints in a later migration after call_records exists, OR omit FKs and enforce in application layer (since call_records is partitioned, cross-partition FKs are not supported by PostgreSQL anyway).

**Step 1:** Write all 3 migration files. For call_tags.call_id and appointments.call_id, use plain UUID columns (no FK constraint — PostgreSQL does not support FK references to partitioned tables).

**Step 2: Commit**

```bash
git add migrations/iiz/06*.sql
git commit -m "feat(iiz): add analytics migrations — tags, scoring, reports, daily summary"
```

---

## Task 9: Compliance & Trust Center Migrations (Shard 08)

**Files:**
- Create: `migrations/iiz/070_compliance_business.sql`
- Create: `migrations/iiz/071_compliance_registrations.sql`
- Create: `migrations/iiz/072_compliance_requirements.sql`

**Column reference (from data dictionary shard 08):**

| Table | Key Details |
|-------|-------------|
| business_info | account_id (UQ — singleton), legal_business_name, dba, ein (TEXT — encrypted at app), industry, full address fields, phone, email, website |
| authorized_contacts | business_info_id FK CASCADE, name, title, phone, email, is_primary |
| a2p_campaigns | campaign_name, brand_name, use_case, description, sample_messages, opt_in/out descriptions, assigned_numbers, max_numbers, monthly_cost (NUMERIC(10,2)), carrier, status (compliance_status enum), rejection_reason, dlc_campaign_id, submitted/approved_at |
| toll_free_registrations | business_name, contact fields, use_case, use_case_description, monthly_volume, toll_free_numbers (JSONB), status (compliance_status enum), rejection_reason, submitted/approved_at |
| voice_registrations | business_name (snapshot), ein (encrypted), full address, status (compliance_status), attestation_level (enum), last_verified_at, next_verification_due (DATE) |
| voice_registration_history | registration_id FK CASCADE, event_date (DATE), event_type, old_status, new_status, notes — append-only |
| caller_id_cnam | number, tracking_number_id FK nullable, current_cnam (15 char), requested_cnam, status, last_updated_at |
| compliance_requirements | country, requirement_name, requirement_description, status (compliance_status), documentation_url, due_date, completed_at |
| compliance_applications | application_name, application_type, country, status (compliance_status), submitted/reviewed/expires_at, rejection_reason, external_reference_id |
| compliance_addresses | label, full address, is_verified, verification_method, verified_at |
| port_requests | numbers_to_port (JSONB), personal fields, full billing address, authorized_signature, status (compliance_status), submitted/completed_at, rejection_reason |

**Step 1:** Write all 3 migration files.

**Step 2: Commit**

```bash
git add migrations/iiz/07*.sql
git commit -m "feat(iiz): add compliance migrations — business info, registrations, trust center"
```

---

## Task 10: Event Log Migrations — Partitioned Tables (Shard 01)

The highest-volume tables. All partitioned by month.

**Files:**
- Create: `migrations/iiz/100_call_records.sql`
- Create: `migrations/iiz/101_call_flow_events.sql`
- Create: `migrations/iiz/102_call_supplementary.sql`
- Create: `migrations/iiz/103_text_records.sql`
- Create: `migrations/iiz/104_other_activity_records.sql`
- Create: `migrations/iiz/105_agent_state_log.sql`
- Create: `migrations/iiz/106_webhook_deliveries.sql`
- Create: `migrations/iiz/107_api_log_entries.sql`
- Create: `migrations/iiz/108_other_event_logs.sql`

**Step 1: Write call_records — the most critical table**

`migrations/iiz/100_call_records.sql`:
```sql
-- Call Records: immutable CDR (partitioned by month on started_at)
-- This is the highest-volume table. Never UPDATEd after INSERT.
-- Mutable annotations split into call_annotations (1:1).

CREATE TABLE iiz.call_records (
    id                      UUID            NOT NULL,
    account_id              UUID            NOT NULL,
    call_id                 TEXT            NOT NULL,
    caller_phone            TEXT,
    callee_phone            TEXT,
    direction               iiz.call_direction NOT NULL,
    status                  iiz.call_status NOT NULL,
    source_id               UUID,
    source_number_id        UUID,
    agent_id                UUID,
    queue_id                UUID,
    started_at              TIMESTAMPTZ     NOT NULL,
    answered_at             TIMESTAMPTZ,
    ended_at                TIMESTAMPTZ,
    duration_secs           INTEGER         NOT NULL DEFAULT 0,
    ring_duration_secs      INTEGER         NOT NULL DEFAULT 0,
    hold_duration_secs      INTEGER         NOT NULL DEFAULT 0,
    recording_url           TEXT,
    has_audio               BOOLEAN         NOT NULL DEFAULT false,
    is_first_time_caller    BOOLEAN         NOT NULL DEFAULT false,
    location                TEXT,
    automation_id           UUID,
    -- Snapshot denormalized fields (copied at write time for query perf)
    source_name             TEXT,
    agent_name              TEXT,
    queue_name              TEXT,
    created_at              TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ,

    PRIMARY KEY (id, started_at)
) PARTITION BY RANGE (started_at);

-- Indexes (inherited by all partitions)
CREATE INDEX idx_cr_account_started ON iiz.call_records (account_id, started_at DESC);
CREATE INDEX idx_cr_caller ON iiz.call_records (caller_phone);
CREATE INDEX idx_cr_callee ON iiz.call_records (callee_phone);
CREATE INDEX idx_cr_source ON iiz.call_records (source_id) WHERE source_id IS NOT NULL;
CREATE INDEX idx_cr_agent ON iiz.call_records (agent_id) WHERE agent_id IS NOT NULL;
CREATE INDEX idx_cr_queue ON iiz.call_records (queue_id) WHERE queue_id IS NOT NULL;
CREATE INDEX idx_cr_status ON iiz.call_records (account_id, status);
CREATE INDEX idx_cr_call_id ON iiz.call_records (call_id);

-- Auto-partition trigger
CREATE TRIGGER create_partition_call_records
    BEFORE INSERT ON iiz.call_records
    FOR EACH ROW EXECUTE FUNCTION iiz.create_monthly_partition('started_at');

-- Call Annotations: mutable overlay (1:1 with call_records).
-- Agent scoring, tagging, and outcome tracking.
-- NOT partitioned — low write volume, always joined by call_id.
CREATE TABLE iiz.call_annotations (
    call_id         UUID            NOT NULL PRIMARY KEY,
    account_id      UUID            NOT NULL,
    score           INTEGER,
    converted       BOOLEAN,
    outcome         TEXT,
    reporting_tag   TEXT,
    category        TEXT,
    appointment_set BOOLEAN,
    notes           TEXT,
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT now(),
    updated_by_id   UUID,
    deleted_at      TIMESTAMPTZ
);

CREATE INDEX idx_ca_account ON iiz.call_annotations (account_id);
CREATE INDEX idx_ca_outcome ON iiz.call_annotations (outcome) WHERE outcome IS NOT NULL;

SELECT iiz.add_updated_at_trigger('call_annotations');
```

**Step 2: Write remaining event log migrations**

Each partitioned table follows the same pattern:
- `PARTITION BY RANGE (<timestamp_column>)`
- Composite PK including the partition key: `PRIMARY KEY (id, <timestamp_column>)`
- Auto-partition trigger calling `iiz.create_monthly_partition('<column>')`
- No `updated_at` (immutable), no notify trigger (event log, not config)
- Indexes on account_id + partition key, plus entity-specific lookups

**Column references:**

| Table | Partition Key | Key Columns |
|-------|--------------|-------------|
| call_flow_events | occurred_at | call_id, event_type (TEXT), occurred_at, detail |
| call_visitor_sessions | created_at | call_id (UQ), ip_address, device, browser, os, referrer, landing_page, keywords, utm_* fields, visit_duration_secs, pages_viewed |
| call_transcription_segments | created_at | call_id, segment_index, timestamp_offset_secs, speaker (enum), content (TEXT), confidence (REAL) |
| call_ai_summaries | generated_at | call_id, summary_type (enum), content (TEXT), model, UQ(call_id, summary_type, generated_at) |
| call_keyword_hits | created_at | call_id, keyword_id (UUID), timestamp_offset_secs, speaker (enum) — NOT partitioned (sparse) |
| text_records | sent_at | contact_phone, tracking_number_id, direction (enum), preview, status, sent_at |
| text_messages | sent_at | contact_phone, tracking_number_id, call_id nullable, direction (enum), body (TEXT), status, sent_at |
| form_records | submitted_at | contact_name/phone/email, form_name, source, tracking_number, form_data (JSONB), status, submitted_at — NOT partitioned |
| chat_records | started_at | visitor_name/detail, channel (enum), message_count, agent_id, widget_id, status, duration_secs, started/ended_at — NOT partitioned |
| fax_records | sent_at | from/to_number, direction (enum), pages, status, document_url, sent_at — NOT partitioned |
| video_records | started_at | participant_name/email, host_agent_id, platform, has_recording, recording_url, duration_secs, started/ended_at — NOT partitioned |
| export_records | created_at | name, export_type, format (enum), date_range, record_count, status, download_url, requested_by_id, filters_applied (JSONB), completed_at — NOT partitioned |
| agent_state_log | changed_at | agent_id, status (enum), changed_at, duration_secs, reason — PARTITIONED |
| webhook_deliveries | delivered_at | webhook_id, event_type, payload (JSONB), http_status_code, response_body, status, attempt_number, delivered_at — PARTITIONED |
| api_log_entries | timestamp | source, method, endpoint, request_headers/body (JSONB), response_code, response_body (JSONB), response_size_bytes, duration_ms, activity_description, error_message — PARTITIONED |
| notifications | created_at | user_id, event_type, title, body, entity_type, entity_id, is_read — NOT partitioned |
| monitoring_events | started_at | session_id, call_id nullable, monitor_user_id, monitored_agent_id nullable, event_type, monitor_mode (enum), started/ended_at, duration_secs — NOT partitioned |
| voice_registration_history | event_date | registration_id FK, event_date, event_type, old/new_status, notes — NOT partitioned (in shard 08, already created) |

**Partitioned (8 tables):** call_records, call_flow_events, call_transcription_segments, call_ai_summaries, text_messages, agent_state_log, webhook_deliveries, api_log_entries

**Not partitioned (10 tables):** call_annotations, call_visitor_sessions, call_keyword_hits, text_records, form_records, chat_records, fax_records, video_records, export_records, notifications, monitoring_events

**Step 3: Commit**

```bash
git add migrations/iiz/10*.sql
git commit -m "feat(iiz): add event log migrations — call records, transcription, activity records"
```

---

## Task 11: Ephemeral & System Migrations (Shard 09)

**Files:**
- Create: `migrations/iiz/120_ephemeral_tables.sql`
- Create: `migrations/iiz/121_knowledge_embeddings.sql`

**Step 1: Write UNLOGGED ephemeral tables**

`migrations/iiz/120_ephemeral_tables.sql`:
```sql
-- Ephemeral real-time state: UNLOGGED tables + moka cache.
-- UNLOGGED skips WAL for ~2x write performance.
-- Data survives normal restarts but lost on crash (acceptable — rebuilt from SIP state).

CREATE UNLOGGED TABLE iiz.active_calls (
    id                  UUID            NOT NULL PRIMARY KEY,
    account_id          UUID            NOT NULL,
    call_id             TEXT            NOT NULL UNIQUE,
    caller_name         TEXT,
    caller_number       TEXT,
    callee_number       TEXT,
    agent_id            UUID,
    queue_id            UUID,
    source_id           UUID,
    tracking_number_id  UUID,
    direction           iiz.call_direction NOT NULL,
    status              iiz.active_call_status NOT NULL DEFAULT 'ringing',
    started_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    answered_at         TIMESTAMPTZ,
    is_monitored        BOOLEAN         NOT NULL DEFAULT false,
    monitor_mode        iiz.monitor_mode,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX idx_ac_account ON iiz.active_calls (account_id);
CREATE INDEX idx_ac_agent ON iiz.active_calls (agent_id) WHERE agent_id IS NOT NULL;

SELECT iiz.add_updated_at_trigger('active_calls');

-- Presence: one row per agent/identity, updated in-place.
CREATE UNLOGGED TABLE iiz.presence (
    identity            TEXT            NOT NULL PRIMARY KEY,
    account_id          UUID,
    user_id             UUID,
    status              iiz.agent_status NOT NULL DEFAULT 'offline',
    note                TEXT,
    activity            TEXT,
    current_call_id     UUID,
    last_updated        TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX idx_presence_account ON iiz.presence (account_id) WHERE account_id IS NOT NULL;

-- Locations: SIP registration bindings with TTL.
CREATE UNLOGGED TABLE iiz.locations (
    id                  UUID            NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID,
    aor                 TEXT            NOT NULL,
    username            TEXT,
    realm               TEXT,
    destination         TEXT            NOT NULL,
    expires             TIMESTAMPTZ     NOT NULL,
    user_agent          TEXT,
    supports_webrtc     BOOLEAN         NOT NULL DEFAULT false,
    source_ip           TEXT,
    source_port         INTEGER,
    transport           iiz.sip_transport,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX idx_loc_aor ON iiz.locations (aor);
CREATE INDEX idx_loc_expires ON iiz.locations (expires);

SELECT iiz.add_updated_at_trigger('locations');

-- Frequency Limits: regular table (needs crash durability for rate limiting).
CREATE TABLE iiz.frequency_limits (
    id                  UUID            NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID            NOT NULL,
    policy_id           TEXT            NOT NULL,
    scope               TEXT            NOT NULL,
    limit_type          TEXT            NOT NULL,
    max_count           INTEGER         NOT NULL,
    current_count       INTEGER         NOT NULL DEFAULT 0,
    window_start        TIMESTAMPTZ,
    window_end          TIMESTAMPTZ,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX idx_fl_policy ON iiz.frequency_limits (policy_id, scope);
CREATE INDEX idx_fl_window ON iiz.frequency_limits (window_end) WHERE window_end IS NOT NULL;

SELECT iiz.add_updated_at_trigger('frequency_limits');

-- Account Variables: key-value config store per account.
CREATE TABLE iiz.account_variables (
    id                  UUID            NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID            NOT NULL REFERENCES iiz.accounts(id) ON DELETE CASCADE,
    name                TEXT            NOT NULL,
    value               TEXT,
    description         TEXT,
    is_secret           BOOLEAN         NOT NULL DEFAULT false,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ,

    CONSTRAINT uq_account_variables UNIQUE (account_id, name)
);

CREATE INDEX idx_av_account ON iiz.account_variables (account_id) WHERE deleted_at IS NULL;

SELECT iiz.add_updated_at_trigger('account_variables');
SELECT iiz.add_notify_trigger('account_variables');
```

**Step 2: Write pgvector embeddings table**

`migrations/iiz/121_knowledge_embeddings.sql`:
```sql
-- Knowledge bank vector embeddings for RAG retrieval.
-- Requires pgvector extension.

CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE iiz.knowledge_bank_embeddings (
    id              UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID        NOT NULL,
    document_id     UUID        NOT NULL,
    chunk_index     INTEGER     NOT NULL,
    chunk_text      TEXT        NOT NULL,
    embedding       vector(1536),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,

    CONSTRAINT uq_kbe_chunk UNIQUE (document_id, chunk_index)
);

CREATE INDEX idx_kbe_account ON iiz.knowledge_bank_embeddings (account_id);
CREATE INDEX idx_kbe_document ON iiz.knowledge_bank_embeddings (document_id);

-- HNSW index for cosine similarity search (faster than IVFFlat, no training needed)
CREATE INDEX idx_kbe_embedding ON iiz.knowledge_bank_embeddings
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);
```

**Step 3: Commit**

```bash
git add migrations/iiz/12*.sql
git commit -m "feat(iiz): add ephemeral tables and pgvector embeddings"
```

---

## Task 12: Cross-Cutting Migrations — RLS, Triggers, Initial Partitions

**Files:**
- Create: `migrations/iiz/130_rls_policies.sql`
- Create: `migrations/iiz/131_notify_triggers.sql`
- Create: `migrations/iiz/132_initial_partitions.sql`

**Step 1: Write RLS policies for all tables**

`migrations/iiz/130_rls_policies.sql`:
```sql
-- Row-Level Security policies for all tables with account_id.
-- Pattern: tenant sees own rows where deleted_at IS NULL.
-- System/admin operations use BYPASSRLS role.

-- Helper to apply standard RLS to a table
CREATE OR REPLACE FUNCTION iiz.apply_rls(table_name TEXT)
RETURNS void AS $$
BEGIN
    EXECUTE format('ALTER TABLE iiz.%I ENABLE ROW LEVEL SECURITY', table_name);
    EXECUTE format('ALTER TABLE iiz.%I FORCE ROW LEVEL SECURITY', table_name);
    EXECUTE format(
        'CREATE POLICY tenant_isolation ON iiz.%I
         FOR ALL
         USING (
             account_id = current_setting(''app.current_account_id'')::uuid
             AND deleted_at IS NULL
         )
         WITH CHECK (
             account_id = current_setting(''app.current_account_id'')::uuid
         )',
        table_name
    );
END;
$$ LANGUAGE plpgsql;

-- Apply RLS to all tables with account_id.
-- accounts table uses a different policy (id = current_account_id, not account_id).

-- Core
SELECT iiz.apply_rls('users');
SELECT iiz.apply_rls('contact_lists');
SELECT iiz.apply_rls('contact_list_members');
SELECT iiz.apply_rls('blocked_numbers');
SELECT iiz.apply_rls('dnc_entries');
SELECT iiz.apply_rls('dnt_entries');

-- Numbers (shard 03)
SELECT iiz.apply_rls('tracking_sources');
SELECT iiz.apply_rls('receiving_numbers');
SELECT iiz.apply_rls('number_pools');
SELECT iiz.apply_rls('number_pool_members');
SELECT iiz.apply_rls('call_settings');
SELECT iiz.apply_rls('tracking_numbers');
SELECT iiz.apply_rls('text_numbers');
SELECT iiz.apply_rls('target_numbers');

-- Routing (shard 04)
SELECT iiz.apply_rls('schedules');
SELECT iiz.apply_rls('schedule_holidays');
SELECT iiz.apply_rls('voice_menus');
SELECT iiz.apply_rls('voice_menu_options');
SELECT iiz.apply_rls('queues');
SELECT iiz.apply_rls('queue_agents');
SELECT iiz.apply_rls('smart_routers');
SELECT iiz.apply_rls('smart_router_rules');
SELECT iiz.apply_rls('geo_routers');
SELECT iiz.apply_rls('geo_router_rules');
SELECT iiz.apply_rls('routing_tables');
SELECT iiz.apply_rls('routing_table_routes');
SELECT iiz.apply_rls('agent_scripts');
SELECT iiz.apply_rls('voicemail_boxes');
SELECT iiz.apply_rls('voicemail_messages');

-- Automation (shard 05)
SELECT iiz.apply_rls('workflows');
SELECT iiz.apply_rls('workflow_nodes');
SELECT iiz.apply_rls('workflow_edges');
SELECT iiz.apply_rls('triggers');
SELECT iiz.apply_rls('trigger_conditions');
SELECT iiz.apply_rls('trigger_actions');
SELECT iiz.apply_rls('lambdas');
SELECT iiz.apply_rls('lambda_env_vars');
SELECT iiz.apply_rls('webhooks');
SELECT iiz.apply_rls('webhook_subscriptions');
SELECT iiz.apply_rls('bulk_messages');
SELECT iiz.apply_rls('lead_reactor_configs');
SELECT iiz.apply_rls('lead_reactor_actions');
SELECT iiz.apply_rls('smart_dialer_configs');
SELECT iiz.apply_rls('form_reactor_entries');
SELECT iiz.apply_rls('reminders');
SELECT iiz.apply_rls('keyword_spotting_configs');
SELECT iiz.apply_rls('keyword_spotting_keywords');
SELECT iiz.apply_rls('keyword_spotting_numbers');
SELECT iiz.apply_rls('chat_widgets');

-- AI (shard 06)
SELECT iiz.apply_rls('ask_ai_configs');
SELECT iiz.apply_rls('summary_configs');
SELECT iiz.apply_rls('knowledge_banks');
SELECT iiz.apply_rls('knowledge_bank_documents');
SELECT iiz.apply_rls('voice_ai_agents');
SELECT iiz.apply_rls('chat_ai_agents');
SELECT iiz.apply_rls('chat_ai_configs');
SELECT iiz.apply_rls('dialogflow_configs');

-- Analytics (shard 07)
SELECT iiz.apply_rls('tags');
SELECT iiz.apply_rls('call_tags');
SELECT iiz.apply_rls('scoring_configs');
SELECT iiz.apply_rls('appointments');
SELECT iiz.apply_rls('custom_reports');
SELECT iiz.apply_rls('notification_rules');
SELECT iiz.apply_rls('call_daily_summary');

-- Compliance (shard 08)
SELECT iiz.apply_rls('business_info');
SELECT iiz.apply_rls('authorized_contacts');
SELECT iiz.apply_rls('a2p_campaigns');
SELECT iiz.apply_rls('toll_free_registrations');
SELECT iiz.apply_rls('voice_registrations');
SELECT iiz.apply_rls('voice_registration_history');
SELECT iiz.apply_rls('caller_id_cnam');
SELECT iiz.apply_rls('compliance_requirements');
SELECT iiz.apply_rls('compliance_applications');
SELECT iiz.apply_rls('compliance_addresses');
SELECT iiz.apply_rls('port_requests');

-- Event logs (shard 01)
SELECT iiz.apply_rls('call_records');
SELECT iiz.apply_rls('call_annotations');
SELECT iiz.apply_rls('call_flow_events');
SELECT iiz.apply_rls('call_visitor_sessions');
SELECT iiz.apply_rls('call_transcription_segments');
SELECT iiz.apply_rls('call_ai_summaries');
SELECT iiz.apply_rls('call_keyword_hits');
SELECT iiz.apply_rls('text_records');
SELECT iiz.apply_rls('text_messages');
SELECT iiz.apply_rls('form_records');
SELECT iiz.apply_rls('chat_records');
SELECT iiz.apply_rls('fax_records');
SELECT iiz.apply_rls('video_records');
SELECT iiz.apply_rls('export_records');
SELECT iiz.apply_rls('agent_state_log');
SELECT iiz.apply_rls('webhook_deliveries');
SELECT iiz.apply_rls('api_log_entries');
SELECT iiz.apply_rls('notifications');
SELECT iiz.apply_rls('monitoring_events');

-- System (shard 09)
SELECT iiz.apply_rls('account_variables');
SELECT iiz.apply_rls('frequency_limits');
SELECT iiz.apply_rls('knowledge_bank_embeddings');

-- Accounts: special policy (id = current_account_id, not account_id)
ALTER TABLE iiz.accounts ENABLE ROW LEVEL SECURITY;
ALTER TABLE iiz.accounts FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON iiz.accounts
    FOR ALL
    USING (
        id = current_setting('app.current_account_id')::uuid
        AND deleted_at IS NULL
    )
    WITH CHECK (
        id = current_setting('app.current_account_id')::uuid
    );

-- Ephemeral tables: RLS on active_calls and presence (account_id nullable on some)
SELECT iiz.apply_rls('active_calls');
-- presence and locations have nullable account_id, skip RLS for now
```

**Step 2: Write LISTEN/NOTIFY triggers for all config tables**

`migrations/iiz/131_notify_triggers.sql`:
```sql
-- Apply NOTIFY triggers to all config tables.
-- These fire on INSERT/UPDATE/DELETE and publish to 'iiz_config_changed' channel.
-- The application listens and invalidates moka cache entries.

-- Note: Some tables already have notify triggers from their creation migrations.
-- This migration adds them to any config tables that don't have them yet.
-- The add_notify_trigger function is idempotent-safe via CREATE TRIGGER IF NOT EXISTS
-- (PG 14+ only). For older versions, wrap in DO block with exception handling.

DO $$
DECLARE
    tbl TEXT;
    config_tables TEXT[] := ARRAY[
        -- Shard 03
        'tracking_sources', 'receiving_numbers', 'number_pools', 'number_pool_members',
        'call_settings', 'tracking_numbers', 'text_numbers', 'target_numbers',
        -- Shard 04
        'schedules', 'schedule_holidays', 'voice_menus', 'voice_menu_options',
        'queues', 'queue_agents', 'smart_routers', 'smart_router_rules',
        'geo_routers', 'geo_router_rules', 'routing_tables', 'routing_table_routes',
        'agent_scripts', 'voicemail_boxes',
        -- Shard 05
        'workflows', 'workflow_nodes', 'workflow_edges',
        'triggers', 'trigger_conditions', 'trigger_actions',
        'lambdas', 'lambda_env_vars',
        'webhooks', 'webhook_subscriptions',
        'lead_reactor_configs', 'lead_reactor_actions',
        'smart_dialer_configs', 'form_reactor_entries',
        'keyword_spotting_configs', 'keyword_spotting_keywords', 'keyword_spotting_numbers',
        'chat_widgets',
        -- Shard 06
        'ask_ai_configs', 'summary_configs', 'knowledge_banks', 'knowledge_bank_documents',
        'voice_ai_agents', 'chat_ai_agents', 'chat_ai_configs', 'dialogflow_configs',
        -- Shard 07
        'tags', 'scoring_configs', 'notification_rules',
        -- Shard 09
        'account_variables'
    ];
BEGIN
    FOREACH tbl IN ARRAY config_tables
    LOOP
        -- Skip if trigger already exists
        IF NOT EXISTS (
            SELECT 1 FROM pg_trigger
            WHERE tgname = 'notify_change'
              AND tgrelid = format('iiz.%I', tbl)::regclass
        ) THEN
            PERFORM iiz.add_notify_trigger(tbl);
        END IF;
    END LOOP;
END $$;
```

**Step 3: Write initial partition creation**

`migrations/iiz/132_initial_partitions.sql`:
```sql
-- Pre-create partitions for current month and next month.
-- Belt-and-suspenders alongside the auto-creation trigger.

DO $$
DECLARE
    current_start DATE := date_trunc('month', now())::date;
    current_end DATE := (current_start + interval '1 month')::date;
    next_start DATE := current_end;
    next_end DATE := (next_start + interval '1 month')::date;
    suffix_current TEXT := to_char(current_start, 'YYYY_MM');
    suffix_next TEXT := to_char(next_start, 'YYYY_MM');
    tbl RECORD;
BEGIN
    -- Partitioned tables and their parent names
    FOR tbl IN
        SELECT unnest(ARRAY[
            'call_records',
            'call_flow_events',
            'call_transcription_segments',
            'call_ai_summaries',
            'text_messages',
            'agent_state_log',
            'webhook_deliveries',
            'api_log_entries'
        ]) AS name
    LOOP
        -- Current month
        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS iiz.%I PARTITION OF iiz.%I
             FOR VALUES FROM (%L) TO (%L)',
            tbl.name || '_' || suffix_current, tbl.name,
            current_start, current_end
        );
        -- Next month
        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS iiz.%I PARTITION OF iiz.%I
             FOR VALUES FROM (%L) TO (%L)',
            tbl.name || '_' || suffix_next, tbl.name,
            next_start, next_end
        );
    END LOOP;
END $$;
```

**Step 4: Commit**

```bash
git add migrations/iiz/13*.sql
git commit -m "feat(iiz): add RLS policies, NOTIFY triggers, and initial partitions"
```

---

## Task 13: Connection Pool Manager

Create the 4-pool connection manager for segregated workloads.

**Files:**
- Create: `src/iiz/pool.rs`
- Modify: `src/iiz/mod.rs`

**Step 1: Write pool manager**

`src/iiz/pool.rs`:
```rust
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use std::str::FromStr;
use std::time::Duration;

/// Four segregated connection pools as specified in the storage architecture.
pub struct IizPools {
    /// CDR inserts, routing lookups — hot path
    pub call_processing: PgPool,
    /// UI/API config reads and writes
    pub api_crud: PgPool,
    /// Exports, bulk sends, aggregation, transcription
    pub background: PgPool,
    /// Dashboard and report queries
    pub reports: PgPool,
}

/// Configuration for pool sizes and timeouts.
pub struct PoolConfig {
    pub database_url: String,
    pub call_processing_max: u32,
    pub api_crud_max: u32,
    pub background_max: u32,
    pub reports_max: u32,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            database_url: String::new(),
            call_processing_max: 20,
            api_crud_max: 10,
            background_max: 5,
            reports_max: 5,
        }
    }
}

impl IizPools {
    pub async fn connect(config: &PoolConfig) -> Result<Self, sqlx::Error> {
        let base_opts = PgConnectOptions::from_str(&config.database_url)?;

        let call_processing = PgPoolOptions::new()
            .max_connections(config.call_processing_max)
            .acquire_timeout(Duration::from_secs(5))
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    sqlx::query("SET search_path = iiz, public; SET timezone = 'UTC';")
                        .execute(&mut *conn)
                        .await?;
                    Ok(())
                })
            })
            .connect_with(base_opts.clone())
            .await?;

        let api_crud = PgPoolOptions::new()
            .max_connections(config.api_crud_max)
            .acquire_timeout(Duration::from_secs(10))
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    sqlx::query("SET search_path = iiz, public; SET timezone = 'UTC'; SET statement_timeout = '30s';")
                        .execute(&mut *conn)
                        .await?;
                    Ok(())
                })
            })
            .connect_with(base_opts.clone())
            .await?;

        let background = PgPoolOptions::new()
            .max_connections(config.background_max)
            .acquire_timeout(Duration::from_secs(30))
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    sqlx::query("SET search_path = iiz, public; SET timezone = 'UTC'; SET statement_timeout = '300s';")
                        .execute(&mut *conn)
                        .await?;
                    Ok(())
                })
            })
            .connect_with(base_opts.clone())
            .await?;

        let reports = PgPoolOptions::new()
            .max_connections(config.reports_max)
            .acquire_timeout(Duration::from_secs(10))
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    sqlx::query("SET search_path = iiz, public; SET timezone = 'UTC'; SET statement_timeout = '60s';")
                        .execute(&mut *conn)
                        .await?;
                    Ok(())
                })
            })
            .connect_with(base_opts)
            .await?;

        Ok(Self {
            call_processing,
            api_crud,
            background,
            reports,
        })
    }

    /// Set the tenant context on a specific pool connection.
    /// Must be called before any query that touches RLS-protected tables.
    pub async fn set_tenant(
        pool: &PgPool,
        account_id: &uuid::Uuid,
    ) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>, sqlx::Error> {
        let mut conn = pool.acquire().await?;
        sqlx::query(&format!(
            "SET app.current_account_id = '{}'",
            account_id
        ))
        .execute(&mut *conn)
        .await?;
        Ok(conn)
    }
}
```

**Step 2: Update module root**

`src/iiz/mod.rs`:
```rust
pub mod migrator;
pub mod pool;
```

**Step 3: Commit**

```bash
git add src/iiz/pool.rs src/iiz/mod.rs
git commit -m "feat(iiz): add 4-pool connection manager with tenant context"
```

---

## Task 14: SeaORM Entity Generation Setup

**Files:**
- Create: `src/iiz/entities/mod.rs`
- Create: `src/iiz/entities/extensions/mod.rs`
- Modify: `src/iiz/mod.rs`
- Modify: `.gitignore` (optional: decide if generated entities are tracked)

**Step 1: Create entity module structure**

`src/iiz/entities/mod.rs`:
```rust
//! SeaORM entities generated from the iiz PostgreSQL schema.
//!
//! Regenerate after schema changes:
//! ```bash
//! sea-orm-cli generate entity \
//!     --database-url "postgres://user:pass@localhost/rustpbx" \
//!     --database-schema iiz \
//!     --output-dir src/iiz/entities/generated \
//!     --with-serde both \
//!     --date-time-crate chrono
//! ```

pub mod extensions;

// Generated entities will be re-exported here after first generation.
// Example (uncomment after running sea-orm-cli):
// pub mod generated;
// pub use generated::*;
```

`src/iiz/entities/extensions/mod.rs`:
```rust
//! Custom extensions to generated SeaORM entities.
//! Business logic, computed fields, and helper methods go here.
//! These files are NOT overwritten by sea-orm-cli regeneration.
```

**Step 2: Update module root**

`src/iiz/mod.rs`:
```rust
pub mod entities;
pub mod migrator;
pub mod pool;
```

**Step 3: Install sea-orm-cli (if not already installed)**

```bash
cargo install sea-orm-cli
```

**Step 4: Document the generation command**

The actual generation happens AFTER migrations are applied to a live database:

```bash
# 1. Start PostgreSQL (local or Docker)
# 2. Apply migrations:
#    cargo run -- --migrate-iiz  (or however we wire it up)
# 3. Generate entities:
sea-orm-cli generate entity \
    --database-url "postgres://user:pass@localhost/rustpbx" \
    --database-schema iiz \
    --output-dir src/iiz/entities/generated \
    --with-serde both \
    --date-time-crate chrono
# 4. Verify and commit generated files
```

**Step 5: Commit**

```bash
git add src/iiz/
git commit -m "feat(iiz): add SeaORM entity module structure"
```

---

## Task 15: Integration Test — Full Migration Smoke Test

**Files:**
- Create: `tests/test_iiz_migrations.py`

**Prerequisites:** A running PostgreSQL instance. Use Docker for CI:
```bash
docker run -d --name iiz-test-pg -e POSTGRES_PASSWORD=test -e POSTGRES_DB=iiz_test -p 5433:5432 postgres:16
```

**Step 1: Write integration test**

`tests/test_iiz_migrations.py`:
```python
"""
Integration test: apply all iiz migrations to a fresh PostgreSQL database
and verify the schema is correct.

Requires: PostgreSQL running on localhost:5433 (or set IIZ_TEST_DATABASE_URL)
"""
import os
import subprocess
import psycopg2
import pytest

DATABASE_URL = os.environ.get(
    "IIZ_TEST_DATABASE_URL",
    "postgresql://postgres:test@localhost:5433/iiz_test"
)

@pytest.fixture(scope="session")
def db_conn():
    """Connect to test database and apply migrations."""
    conn = psycopg2.connect(DATABASE_URL)
    conn.autocommit = True

    # Drop and recreate iiz schema for clean test
    cur = conn.cursor()
    cur.execute("DROP SCHEMA IF EXISTS iiz CASCADE")
    cur.execute("CREATE SCHEMA iiz")
    conn.commit()

    # Apply migrations via the Rust binary or direct SQL
    # For now, apply SQL files directly
    migrations_dir = os.path.join(os.path.dirname(__file__), "..", "migrations", "iiz")
    sql_files = sorted([
        f for f in os.listdir(migrations_dir)
        if f.endswith(".sql") and not f.endswith("_down.sql")
    ])

    for sql_file in sql_files:
        path = os.path.join(migrations_dir, sql_file)
        with open(path, "r") as f:
            sql = f.read()
        try:
            cur.execute(sql)
            conn.commit()
        except Exception as e:
            conn.rollback()
            pytest.fail(f"Migration {sql_file} failed: {e}")

    yield conn
    conn.close()


def test_schema_exists(db_conn):
    cur = db_conn.cursor()
    cur.execute("""
        SELECT schema_name FROM information_schema.schemata
        WHERE schema_name = 'iiz'
    """)
    assert cur.fetchone() is not None


def test_table_count(db_conn):
    """Verify we have approximately 80+ tables."""
    cur = db_conn.cursor()
    cur.execute("""
        SELECT COUNT(*) FROM information_schema.tables
        WHERE table_schema = 'iiz'
          AND table_type = 'BASE TABLE'
    """)
    count = cur.fetchone()[0]
    # ~80 tables + schema_migrations + partitions
    assert count >= 80, f"Expected 80+ tables, got {count}"


def test_partitioned_tables_exist(db_conn):
    """Verify the 8 partitioned parent tables exist."""
    cur = db_conn.cursor()
    partitioned = [
        "call_records", "call_flow_events", "call_transcription_segments",
        "call_ai_summaries", "text_messages", "agent_state_log",
        "webhook_deliveries", "api_log_entries"
    ]
    for table in partitioned:
        cur.execute("""
            SELECT 1 FROM pg_partitioned_table pt
            JOIN pg_class c ON c.oid = pt.partrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'iiz' AND c.relname = %s
        """, (table,))
        assert cur.fetchone() is not None, f"{table} is not partitioned"


def test_rls_enabled(db_conn):
    """Verify RLS is enabled on key tables."""
    cur = db_conn.cursor()
    tables_with_rls = ["users", "call_records", "queues", "tracking_numbers"]
    for table in tables_with_rls:
        cur.execute("""
            SELECT relrowsecurity, relforcerowsecurity
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'iiz' AND c.relname = %s
        """, (table,))
        row = cur.fetchone()
        assert row is not None, f"Table {table} not found"
        assert row[0] is True, f"RLS not enabled on {table}"
        assert row[1] is True, f"RLS not forced on {table}"


def test_unlogged_tables(db_conn):
    """Verify ephemeral tables are UNLOGGED."""
    cur = db_conn.cursor()
    unlogged = ["active_calls", "presence", "locations"]
    for table in unlogged:
        cur.execute("""
            SELECT relpersistence FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'iiz' AND c.relname = %s
        """, (table,))
        row = cur.fetchone()
        assert row is not None, f"Table {table} not found"
        assert row[0] == 'u', f"{table} is not UNLOGGED (persistence={row[0]})"


def test_uuid_primary_keys(db_conn):
    """Verify key tables use UUID primary keys."""
    cur = db_conn.cursor()
    tables = ["accounts", "users", "tracking_numbers", "queues"]
    for table in tables:
        cur.execute("""
            SELECT data_type FROM information_schema.columns
            WHERE table_schema = 'iiz'
              AND table_name = %s
              AND column_name = 'id'
        """, (table,))
        row = cur.fetchone()
        assert row is not None, f"No 'id' column on {table}"
        assert row[0] == 'uuid', f"{table}.id is {row[0]}, expected uuid"


def test_deleted_at_on_all_tables(db_conn):
    """Verify every table has a deleted_at column (soft delete everywhere)."""
    cur = db_conn.cursor()
    cur.execute("""
        SELECT t.table_name
        FROM information_schema.tables t
        WHERE t.table_schema = 'iiz'
          AND t.table_type = 'BASE TABLE'
          AND t.table_name != 'schema_migrations'
          AND NOT EXISTS (
              SELECT 1 FROM information_schema.columns c
              WHERE c.table_schema = 'iiz'
                AND c.table_name = t.table_name
                AND c.column_name = 'deleted_at'
          )
    """)
    missing = [row[0] for row in cur.fetchall()]
    # Filter out partition children (they inherit deleted_at from parent)
    missing = [t for t in missing if not any(
        t.startswith(p + "_2") for p in [
            "call_records", "call_flow_events", "call_transcription_segments",
            "call_ai_summaries", "text_messages", "agent_state_log",
            "webhook_deliveries", "api_log_entries"
        ]
    )]
    assert missing == [], f"Tables missing deleted_at: {missing}"


def test_initial_partitions_created(db_conn):
    """Verify current and next month partitions exist."""
    cur = db_conn.cursor()
    cur.execute("""
        SELECT COUNT(*) FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'iiz'
          AND c.relname LIKE 'call_records_20%%'
          AND c.relkind = 'r'
    """)
    count = cur.fetchone()[0]
    assert count >= 2, f"Expected at least 2 call_records partitions, got {count}"
```

**Step 2: Run integration test**

```bash
# Start test PostgreSQL
docker run -d --name iiz-test-pg -e POSTGRES_PASSWORD=test -e POSTGRES_DB=iiz_test -p 5433:5432 postgres:16

# Wait for PG to be ready
sleep 3

# Run tests
cd /c/Development/RustPBX && python -m pytest tests/test_iiz_migrations.py -v
```

**Step 3: Commit**

```bash
git add tests/test_iiz_migrations.py
git commit -m "test(iiz): add migration integration test suite"
```

---

## Execution Order & Parallelism

```
Task 1  (Migration runner)     ─── sequential (foundation)
Task 2  (Schema + functions)   ─── sequential (foundation)
Task 3  (Accounts + users)     ─── sequential (FK target for everything)
                                    │
                ┌───────────────────┼───────────────────┐
                ▼                   ▼                   ▼
Task 4  (Numbers)          Task 5  (Routing)     Task 9  (Compliance)
                │                   │
                ▼                   ▼
Task 6  (Automation)       Task 7  (AI Tools)
                │                   │
                └─────────┬─────────┘
                          ▼
                Task 8  (Analytics) ─── needs tags, scoring
                          │
                          ▼
                Task 10 (Event Logs) ─── needs all FK targets
                          │
                          ▼
                Task 11 (Ephemeral)
                          │
                          ▼
                Task 12 (RLS + Triggers + Partitions)
                          │
                          ▼
                Task 13 (Connection pools) ─── can parallel with 14
                Task 14 (SeaORM setup)     ─── can parallel with 13
                          │
                          ▼
                Task 15 (Integration test) ─── final verification
```

**Tasks 4, 5, 9 can run in parallel** (independent shards, no cross-FK dependencies).
**Tasks 6, 7 can run in parallel** (both depend on shard 03/04 but not on each other).
**Tasks 13, 14 can run in parallel** (Rust code, no SQL dependencies).

---

## Verification Checklist

After all tasks are complete:

- [ ] `cargo test iiz::migrator` — migration runner unit tests pass
- [ ] `cargo build` — project compiles with new iiz module
- [ ] All migration SQL files are syntactically valid
- [ ] Integration test passes against real PostgreSQL
- [ ] Table count matches expected (~80 base tables)
- [ ] All 8 partitioned tables verified
- [ ] RLS enabled and forced on all tenant tables
- [ ] UNLOGGED tables verified for ephemeral state
- [ ] UUID PKs on all tables
- [ ] `deleted_at` column on every table
- [ ] NOTIFY triggers on all config tables
- [ ] Initial partitions for current + next month exist
- [ ] `sea-orm-cli generate entity` produces valid Rust code
