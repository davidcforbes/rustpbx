//! Four segregated Diesel-async connection pools per the storage architecture.
//!
//! Each pool sets `search_path`, `timezone`, and `statement_timeout` on connect.
//! The `set_tenant()` method sets the RLS context variable before queries.

use diesel_async::pooled_connection::bb8::{Pool, PooledConnection};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;

pub type PgPool = Pool<AsyncPgConnection>;

/// Four segregated connection pools per the storage architecture.
pub struct IizPools {
    /// CDR inserts, routing lookups — hot path (max 20, 5s timeout)
    pub call_processing: PgPool,
    /// UI/API config reads and writes (max 10, 30s statement timeout)
    pub api_crud: PgPool,
    /// Exports, bulk sends, aggregation, transcription (max 5, 300s statement timeout)
    pub background: PgPool,
    /// Dashboard and report queries (max 5, 60s statement timeout)
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

/// Build a single pool with the given max connections.
async fn build_pool(database_url: &str, max_connections: u32) -> anyhow::Result<PgPool> {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(max_connections)
        .build(manager)
        .await?;
    Ok(pool)
}

impl IizPools {
    pub async fn connect(config: &PoolConfig) -> anyhow::Result<Self> {
        let call_processing = build_pool(&config.database_url, config.call_processing_max).await?;
        let api_crud = build_pool(&config.database_url, config.api_crud_max).await?;
        let background = build_pool(&config.database_url, config.background_max).await?;
        let reports = build_pool(&config.database_url, config.reports_max).await?;

        // Initialize each pool's first connection with search_path and timezone.
        for (pool, timeout) in [
            (&call_processing, "5s"),
            (&api_crud, "30s"),
            (&background, "300s"),
            (&reports, "60s"),
        ] {
            let mut conn = pool.get().await?;
            diesel::sql_query("SET search_path = iiz, public")
                .execute(&mut *conn)
                .await?;
            diesel::sql_query("SET timezone = 'UTC'")
                .execute(&mut *conn)
                .await?;
            diesel::sql_query(&format!("SET statement_timeout = '{}'", timeout))
                .execute(&mut *conn)
                .await?;
        }

        Ok(Self {
            call_processing,
            api_crud,
            background,
            reports,
        })
    }

    /// Set the tenant context on a connection.
    /// Must be called before any query touching RLS-protected tables.
    pub async fn set_tenant<'a>(
        pool: &'a PgPool,
        account_id: &uuid::Uuid,
    ) -> anyhow::Result<PooledConnection<'a, AsyncPgConnection>> {
        let mut conn = pool.get().await?;
        diesel::sql_query(&format!(
            "SET app.current_account_id = '{}'",
            account_id
        ))
        .execute(&mut *conn)
        .await?;
        Ok(conn)
    }
}
