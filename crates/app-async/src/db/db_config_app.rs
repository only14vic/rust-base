use {
    app_base::prelude::*,
    core::{
        ptr::null_mut,
        sync::atomic::{AtomicBool, AtomicPtr, Ordering}
    },
    sqlx::{Acquire, Pool, Postgres, Row},
    std::sync::Arc,
    tokio::task::spawn_local
};

#[derive(Debug)]
pub struct DbConfigApp {
    db_pool: Arc<Pool<Postgres>>,
    loaded: AtomicBool,
    config: AtomicPtr<IndexMap<String, String>>
}

impl DbConfigApp {
    pub fn new(db_pool: &Arc<Pool<Postgres>>) -> Arc<Self> {
        let this: Arc<_> = Self {
            db_pool: db_pool.clone(),
            loaded: AtomicBool::new(false),
            config: AtomicPtr::new(null_mut())
        }
        .into();

        let this_clone = this.clone();
        spawn_local(async move { this_clone.load().await.unwrap() });

        this
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.list().get(name).map(String::as_str)
    }

    pub fn list(&self) -> &IndexMap<String, String> {
        while self.config.load(Ordering::Relaxed).is_null() {}
        unsafe { &*self.config.load(Ordering::Relaxed) }
    }

    pub async fn load(&self) -> Void {
        if self.loaded.swap(true, Ordering::SeqCst) == false {
            let mut db_pool = self.db_pool.acquire().await?;
            let db_conn = db_pool.acquire().await?;

            let items = sqlx::query(
                "select
                    name,
                    coalesce(value, default_value)
                 from app.config"
            )
            .fetch_all(db_conn)
            .await?;

            let iter = items
                .iter()
                .map(|r| (r.get::<'_, String, _>(0), r.get::<'_, String, _>(1)));

            let config = IndexMap::from_iter(iter);

            self.config
                .store(Box::leak(config.into()), Ordering::Release);
        } else {
            while self.config.load(Ordering::Relaxed).is_null() {}
        }

        ok()
    }
}
