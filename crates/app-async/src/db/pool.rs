use {
    crate::db::DbConfig,
    app_base::prelude::*,
    futures::lock::Mutex,
    sqlx::{
        pool::{Pool, PoolOptions},
        Database
    },
    std::{
        any::Any,
        collections::HashMap,
        sync::{Arc, LazyLock},
        time::Duration
    }
};

const ERR_INVALID_DOWNCAST_TO_POOL: &str = "Invalid downcast database pool";

static POOLS: LazyLock<Mutex<HashMap<DbConfig, Arc<dyn Any + Send + Sync>>>> =
    LazyLock::new(Default::default);

static DEFAULT_CONFIG: LazyLock<DbConfig> = LazyLock::new(|| {
    let mut config = DbConfig::default();
    config.load_env().unwrap();
    config
});

/// # Gets "sqlx" database pool by config
///
/// If `config` is `None` then default config will be used.
pub async fn db_pool<D: Database>(config: Option<&DbConfig>) -> OkAsync<Arc<Pool<D>>> {
    let config = match config {
        Some(config) => config,
        None => &DEFAULT_CONFIG
    };

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

    let pool: Pool<D> = opts.connect_lazy(&config.url)?;
    let pool: Arc<dyn Any + Send + Sync> = Arc::new(pool);

    pool_ref = pool
        .clone()
        .downcast()
        .map_err(|_| ERR_INVALID_DOWNCAST_TO_POOL)?;
    pools.insert(config.clone(), pool);

    Ok(pool_ref)
}

pub async fn db_pool_reset(config: Option<&DbConfig>) {
    if let Some(config) = config {
        POOLS.lock().await.remove(config);
    } else {
        *POOLS.lock().await = Default::default();
    }
}
