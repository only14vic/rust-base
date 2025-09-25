use {
    crate::db::DbConfig,
    app_base::prelude::*,
    futures::lock::Mutex,
    sqlx::{
        Database,
        pool::{Pool, PoolOptions}
    },
    std::{
        any::Any,
        collections::HashMap,
        sync::{Arc, LazyLock},
        time::Duration
    }
};

const ERR_INVALID_DOWNCAST_TO_POOL: &str = "DB Error: Invalid downcast database pool";

type PoolsMap = HashMap<Arc<DbConfig>, Arc<dyn Any + Send + Sync>>;

static POOLS: LazyLock<Mutex<PoolsMap>> = LazyLock::new(Default::default);

static DEFAULT_CONFIG: LazyLock<Arc<DbConfig>> = LazyLock::new(|| {
    let mut config = DbConfig::default();
    config.load_env();
    config.into()
});

/// # Gets "sqlx" database pool by config
///
/// If `config` is `None` then default config will be used.
pub async fn db_pool<D: Database>(
    config: Option<&Arc<DbConfig>>
) -> OkAsync<Arc<Pool<D>>> {
    let config = config.unwrap_or(&DEFAULT_CONFIG);
    let pool_ref: Arc<Pool<D>>;
    let mut pools = POOLS.lock().await;

    if let Some(item) = pools.get(config).cloned() {
        pool_ref = item.downcast().map_err(|_| ERR_INVALID_DOWNCAST_TO_POOL)?;
        return Ok(pool_ref);
    }

    let opts = PoolOptions::new()
        .min_connections(config.min_conn)
        .max_connections(config.max_conn)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout))
        .idle_timeout(Duration::from_secs(config.idle_timeout))
        .max_lifetime(Duration::from_secs(config.max_lifetime));

    let pool: Pool<D> = opts
        .connect_lazy(&config.url)
        .map_err(|e| format!("DB Error: {e}"))?;
    let pool: Arc<dyn Any + Send + Sync> = Arc::new(pool);

    pool_ref = pool
        .clone()
        .downcast()
        .map_err(|_| ERR_INVALID_DOWNCAST_TO_POOL)?;

    pools.insert(config.clone(), pool);

    Ok(pool_ref)
}

pub async fn db_pool_reset(config: Option<&Arc<DbConfig>>) {
    if let Some(config) = config {
        POOLS.lock().await.remove(config);
    } else {
        *POOLS.lock().await = Default::default();
    }
}
