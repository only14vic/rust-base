use {
    app_base::prelude::*,
    core::{error::Error, mem::ManuallyDrop, pin::Pin, time::Duration},
    futures::executor::block_on,
    sqlx::{
        Pool, Postgres,
        postgres::{PgListener, PgNotification}
    },
    std::sync::Arc,
    tokio::{
        spawn,
        task::{JoinHandle, spawn_blocking},
        time::sleep
    }
};

pub type DbNotifyHandler = Arc<
    dyn Fn(PgNotification) -> Pin<Box<dyn Future<Output = Void> + Send>> + Send + Sync
>;

pub struct DbNotifyListener {
    db_pool: Arc<Pool<Postgres>>,
    channels: Vec<String>,
    handler: DbNotifyHandler
}

impl DbNotifyListener {
    pub fn new(
        channels: impl IntoIterator<Item = impl ToString>,
        db_pool: &Arc<Pool<Postgres>>,
        handler: DbNotifyHandler
    ) -> Self {
        Self {
            db_pool: db_pool.clone(),
            channels: channels.into_iter().map(|c| c.to_string()).collect(),
            handler
        }
    }

    pub async fn start(self) -> JoinHandle<OkAsync<Self>> {
        spawn_blocking(move || block_on(self.run()))
    }

    async fn run(self) -> OkAsync<Self> {
        let db = self.db_pool.clone();

        'main: loop {
            // Wait DB connection is ok
            while db.acquire().await.is_err() {
                sleep(Duration::from_millis(5)).await;
            }

            // Trying create listener
            let listener = match PgListener::connect_with(&db).await {
                Ok(mut listener) => {
                    listener.ignore_pool_close_event(true);

                    if let Err(e) = listener
                        .listen_all(self.channels.iter().map(String::as_str))
                        .await
                    {
                        log::warn!("{e}");
                        continue;
                    }

                    listener
                },
                Err(e) => {
                    log::warn!("{e}");
                    continue;
                }
            };

            Env::is_debug()
                .then(|| log::debug!("Listening db channels: {:?}", &self.channels));

            let mut listener = ManuallyDrop::new(listener);

            // Listening notifies
            'listen: loop {
                match listener.recv().await {
                    Ok(pg_notify) => {
                        let handler = self.handler.clone();
                        spawn(async move {
                            if let Err(e) = handler(pg_notify).await {
                                log::error!("{e}")
                            }
                        });
                    },
                    Err(e) => {
                        log::warn!("{e}");

                        if let Some(err) = e.source()
                            && let Some(err) = err.downcast_ref::<tokio::io::Error>()
                            && err.kind() == tokio::io::ErrorKind::Other
                        {
                            break 'main;
                        }

                        break 'listen;
                    }
                }
            }
        }

        Ok(self)
    }
}
