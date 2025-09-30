use {
    crate::queue::QueueTask,
    app_base::prelude::*,
    core::{str::FromStr, time::Duration},
    futures::future::BoxFuture,
    sqlx::{Executor, Pool, Postgres, postgres::PgNotification, types::Uuid},
    std::sync::Arc,
    tokio::{spawn, task::JoinHandle, time::sleep}
};

pub struct QueueListener {
    db_pool: Arc<Pool<Postgres>>
}

impl QueueListener {
    pub fn new(db_pool: &Arc<Pool<Postgres>>) -> Self {
        Self { db_pool: db_pool.clone() }
    }

    pub async fn start_resend_periodically(&self) -> JoinHandle<()> {
        let db_pool = self.db_pool.clone();
        spawn(async move {
            loop {
                if let Ok(mut conn) = db_pool.acquire().await {
                    let _ = conn
                        .execute("select app.queue_resend()")
                        .await
                        .map_err(|e| log::warn!("{e}"));
                }

                sleep(Duration::from_secs(1)).await;
            }
        })
    }

    pub fn handler(
        &self
    ) -> Arc<dyn Fn(PgNotification) -> BoxFuture<'static, Void> + Send + Sync + 'static>
    {
        let db_pool = self.db_pool.clone();
        Arc::new(move |notify| {
            let db_pool = db_pool.clone();
            async move {
                let mut conn = db_pool.acquire().await?;
                let id = Uuid::from_str(notify.payload())?;

                if let Some(task) = QueueTask::start_process(&id, &mut conn).await? {
                    dbg!(&task);
                    task.finish_process(None, &mut conn).await?;
                }

                ok()
            }
            .into_pin_box()
        })
    }
}
