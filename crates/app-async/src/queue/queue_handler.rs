use {
    crate::{
        db::DbNotifyHandler,
        queue::{QueueTask, QueueTaskHandler}
    },
    app_base::prelude::*,
    core::{str::FromStr, time::Duration},
    futures::FutureExt,
    sqlx::{Acquire, Executor, Pool, Postgres, types::Uuid},
    std::sync::Arc,
    tokio::{spawn, task::JoinHandle, time::sleep}
};

pub struct QueueHandler {
    db_pool: Arc<Pool<Postgres>>,
    handlers: Arc<IndexMap<&'static str, Box<dyn QueueTaskHandler>>>
}

impl QueueHandler {
    pub fn new(
        db_pool: &Arc<Pool<Postgres>>,
        handlers: impl IntoIterator<Item = impl Into<Box<dyn QueueTaskHandler>>>
    ) -> Self {
        Self {
            db_pool: db_pool.clone(),
            handlers: IndexMap::from_iter(handlers.into_iter().map(|t| {
                let task: Box<dyn QueueTaskHandler> = t.into();
                (task.name(), task)
            }))
            .into()
        }
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

    pub fn handler(&self) -> DbNotifyHandler {
        let db_pool = self.db_pool.clone();
        let handlers = self.handlers.clone();

        Arc::new(move |notify| {
            if notify.payload().is_empty() {
                return async { ok() }.into_pin_box();
            }

            let db_pool = db_pool.clone();
            let handlers = handlers.clone();

            async move {
                let id = Uuid::from_str(notify.payload())?;
                let mut db_pool = db_pool.acquire().await?;
                let conn = db_pool.acquire().await?;

                if let Some(task) = QueueTask::start_process(&id, conn).await? {
                    let handler = match handlers.get(task.name.as_str()) {
                        Some(handler) => handler,
                        None => {
                            let error = format!("Undefined task name '{}'", &task.name);
                            task.finish_process(Some(&error), conn).await?;
                            log::error!("{id}: {error}");
                            return ok();
                        }
                    };

                    let error = match handler.handle(&task).await {
                        Ok(..) => None,
                        Err(e) => {
                            log::error!("{id}: {e}");
                            Some(format!("{e}"))
                        }
                    };

                    task.finish_process(error.as_deref(), conn).await?;
                }

                ok()
            }
            .boxed()
        })
    }
}
