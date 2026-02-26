//! Lightweight SQL migration runner for the iiz schema.
//!
//! Reads raw `.sql` files from a `migrations/iiz/` directory and applies them
//! to PostgreSQL in lexicographic filename order. Each migration runs in its
//! own transaction. Applied versions are tracked in `iiz.schema_migrations`.
//!
//! This intentionally avoids SeaORM's migration framework because over half
//! the DDL is PostgreSQL-specific (partitioning, RLS, PL/pgSQL triggers,
//! pgvector). SeaORM is used for runtime queries only, not schema management.

use anyhow::{Context, Result};
use sqlx::PgPool;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// A single migration loaded from a `.sql` file.
#[derive(Debug, Clone)]
pub struct Migration {
    /// Filename without the `.sql` extension, used as the version key.
    pub version: String,
    /// Full filename (e.g. `0001_create_accounts.sql`).
    pub name: String,
    /// Raw SQL contents of the file.
    pub sql: String,
}

/// Reads and applies SQL migrations from a directory.
pub struct Migrator {
    migrations_dir: PathBuf,
}

impl Migrator {
    /// Create a new migrator pointing at the given directory.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            migrations_dir: dir.into(),
        }
    }

    /// Load all forward-migration `.sql` files from `dir`, sorted by filename.
    ///
    /// Files ending in `_down.sql` are excluded (they are rollback companions).
    pub fn load_migrations(dir: &Path) -> Result<Vec<Migration>> {
        let mut migrations = Vec::new();

        if !dir.exists() {
            warn!("migrations directory does not exist: {}", dir.display());
            return Ok(migrations);
        }

        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .with_context(|| format!("failed to read migrations directory: {}", dir.display()))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.ends_with(".sql") && !name.ends_with("_down.sql")
            })
            .collect();

        // Sort by filename for deterministic ordering.
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            let version = name.trim_end_matches(".sql").to_string();
            let sql = std::fs::read_to_string(entry.path())
                .with_context(|| format!("failed to read migration file: {}", entry.path().display()))?;
            migrations.push(Migration { version, name, sql });
        }

        Ok(migrations)
    }

    /// Apply all pending migrations to the database.
    ///
    /// Returns the number of migrations that were applied.
    pub async fn run(&self, pool: &PgPool) -> Result<usize> {
        // Ensure the iiz schema and tracking table exist.
        sqlx::query("CREATE SCHEMA IF NOT EXISTS iiz")
            .execute(pool)
            .await
            .context("failed to create iiz schema")?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS iiz.schema_migrations (
                version TEXT PRIMARY KEY,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .execute(pool)
        .await
        .context("failed to create schema_migrations table")?;

        // Load migration files.
        let migrations = Self::load_migrations(&self.migrations_dir)?;
        if migrations.is_empty() {
            info!("no migration files found in {}", self.migrations_dir.display());
            return Ok(0);
        }

        // Query already-applied versions.
        let applied: HashSet<String> =
            sqlx::query_scalar::<_, String>("SELECT version FROM iiz.schema_migrations")
                .fetch_all(pool)
                .await
                .context("failed to query applied migrations")?
                .into_iter()
                .collect();

        // Filter to pending migrations.
        let pending: Vec<&Migration> = migrations
            .iter()
            .filter(|m| !applied.contains(&m.version))
            .collect();

        if pending.is_empty() {
            info!("all {} migrations already applied", migrations.len());
            return Ok(0);
        }

        info!(
            "{} pending migration(s) out of {} total",
            pending.len(),
            migrations.len()
        );

        let mut count = 0;
        for migration in &pending {
            info!("applying migration: {}", migration.name);

            let mut tx = pool.begin().await.context("failed to begin transaction")?;

            let statements = split_sql_statements(&migration.sql);
            for stmt in &statements {
                sqlx::query(stmt)
                    .execute(&mut *tx)
                    .await
                    .with_context(|| {
                        format!(
                            "failed to execute statement in migration {}: {}",
                            migration.name,
                            &stmt[..stmt.len().min(120)]
                        )
                    })?;
            }

            // Record the version.
            sqlx::query("INSERT INTO iiz.schema_migrations (version) VALUES ($1)")
                .bind(&migration.version)
                .execute(&mut *tx)
                .await
                .with_context(|| {
                    format!("failed to record migration version: {}", migration.version)
                })?;

            tx.commit().await.context("failed to commit migration")?;
            info!("applied migration: {}", migration.name);
            count += 1;
        }

        info!("applied {} migration(s)", count);
        Ok(count)
    }

    /// Roll back a single migration by running its companion `_down.sql` file.
    pub async fn rollback(&self, pool: &PgPool, version: &str) -> Result<()> {
        let down_file = self.migrations_dir.join(format!("{}_down.sql", version));
        if !down_file.exists() {
            anyhow::bail!(
                "rollback file not found: {}",
                down_file.display()
            );
        }

        let sql = std::fs::read_to_string(&down_file)
            .with_context(|| format!("failed to read rollback file: {}", down_file.display()))?;

        info!("rolling back migration: {}", version);

        let mut tx = pool.begin().await.context("failed to begin transaction")?;

        let statements = split_sql_statements(&sql);
        for stmt in &statements {
            sqlx::query(stmt)
                .execute(&mut *tx)
                .await
                .with_context(|| {
                    format!(
                        "failed to execute rollback statement for {}: {}",
                        version,
                        &stmt[..stmt.len().min(120)]
                    )
                })?;
        }

        // Remove the version from the tracking table.
        sqlx::query("DELETE FROM iiz.schema_migrations WHERE version = $1")
            .bind(version)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("failed to remove migration record: {}", version))?;

        tx.commit().await.context("failed to commit rollback")?;
        info!("rolled back migration: {}", version);

        Ok(())
    }
}

/// Split a SQL string into individual statements on semicolons, respecting
/// dollar-quoted strings (`$$...$$`) and single-quoted strings (`'...'`).
///
/// Blank statements and SQL comments (`--` line comments) are excluded from
/// the result.
pub fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements: Vec<String> = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = sql.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        // Skip `--` line comments.
        if ch == '-' && i + 1 < len && chars[i + 1] == '-' {
            // Advance past the rest of the line.
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            // Keep the newline so the current statement stays readable.
            if i < len {
                current.push('\n');
                i += 1;
            }
            continue;
        }

        // Dollar-quoted string: $$ ... $$
        if ch == '$' && i + 1 < len && chars[i + 1] == '$' {
            current.push('$');
            current.push('$');
            i += 2;
            // Read until the closing $$
            while i < len {
                if chars[i] == '$' && i + 1 < len && chars[i + 1] == '$' {
                    current.push('$');
                    current.push('$');
                    i += 2;
                    break;
                }
                current.push(chars[i]);
                i += 1;
            }
            continue;
        }

        // Single-quoted string: '...' (with '' escape)
        if ch == '\'' {
            current.push('\'');
            i += 1;
            while i < len {
                if chars[i] == '\'' {
                    current.push('\'');
                    i += 1;
                    // Escaped quote ('')
                    if i < len && chars[i] == '\'' {
                        current.push('\'');
                        i += 1;
                    } else {
                        break;
                    }
                } else {
                    current.push(chars[i]);
                    i += 1;
                }
            }
            continue;
        }

        // Statement terminator.
        if ch == ';' {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                statements.push(trimmed);
            }
            current.clear();
            i += 1;
            continue;
        }

        current.push(ch);
        i += 1;
    }

    // Trailing statement without a semicolon.
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        statements.push(trimmed);
    }

    statements
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_split_simple_statements() {
        let sql = "CREATE TABLE a (id INT); CREATE TABLE b (id INT);";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 2);
        assert_eq!(stmts[0], "CREATE TABLE a (id INT)");
        assert_eq!(stmts[1], "CREATE TABLE b (id INT)");
    }

    #[test]
    fn test_split_preserves_dollar_quoted_functions() {
        let sql = r#"
CREATE FUNCTION iiz.updated_at() RETURNS trigger AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE t (id INT);
"#;
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 2, "expected 2 statements, got: {:?}", stmts);
        // The function body must be preserved intact (internal semicolons kept).
        assert!(stmts[0].contains("NEW.updated_at = now();"));
        assert!(stmts[0].contains("RETURN NEW;"));
        assert_eq!(stmts[1], "CREATE TABLE t (id INT)");
    }

    #[test]
    fn test_split_preserves_single_quoted_strings() {
        let sql = "INSERT INTO t (name) VALUES ('hello; world'); SELECT 1;";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 2);
        assert!(stmts[0].contains("'hello; world'"));
        assert_eq!(stmts[1], "SELECT 1");
    }

    #[test]
    fn test_split_skips_comments() {
        let sql = "-- This is a comment\nCREATE TABLE a (id INT);\n-- Another comment\nCREATE TABLE b (id INT);";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 2);
        assert!(!stmts[0].contains("--"));
        assert!(!stmts[1].contains("--"));
        assert_eq!(stmts[0], "CREATE TABLE a (id INT)");
        assert_eq!(stmts[1], "CREATE TABLE b (id INT)");
    }

    #[test]
    fn test_load_migrations_sorts_by_filename() {
        let tmp = TempDir::new().unwrap();

        // Write files in reverse order to confirm sorting.
        fs::write(tmp.path().join("0003_indexes.sql"), "CREATE INDEX idx ON t(id);").unwrap();
        fs::write(tmp.path().join("0001_create_schema.sql"), "CREATE SCHEMA iiz;").unwrap();
        fs::write(tmp.path().join("0002_create_tables.sql"), "CREATE TABLE t (id INT);").unwrap();
        // Rollback files should be excluded.
        fs::write(tmp.path().join("0001_create_schema_down.sql"), "DROP SCHEMA iiz;").unwrap();

        let migrations = Migrator::load_migrations(tmp.path()).unwrap();

        assert_eq!(migrations.len(), 3, "expected 3 forward migrations, _down.sql excluded");
        assert_eq!(migrations[0].version, "0001_create_schema");
        assert_eq!(migrations[1].version, "0002_create_tables");
        assert_eq!(migrations[2].version, "0003_indexes");

        // Verify names are full filenames.
        assert_eq!(migrations[0].name, "0001_create_schema.sql");
    }
}
