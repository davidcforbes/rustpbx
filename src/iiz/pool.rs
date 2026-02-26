use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use std::str::FromStr;
use std::time::Duration;

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

    /// Set the tenant context on a pool connection.
    /// Must be called before any query touching RLS-protected tables.
    pub async fn set_tenant(
        pool: &PgPool,
        account_id: &uuid::Uuid,
    ) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>, sqlx::Error> {
        let mut conn = pool.acquire().await?;
        sqlx::query(&format!("SET app.current_account_id = '{}'", account_id))
            .execute(&mut *conn)
            .await?;
        Ok(conn)
    }
}
