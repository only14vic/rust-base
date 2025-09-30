use {app_base::prelude::*, futures::future::BoxFuture, sqlx::postgres::PgNotification};

pub struct QueueListener;

impl QueueListener {
    pub fn handle(notify: PgNotification) -> BoxFuture<'static, Void> {
        Box::pin(async move {
            dbg!(notify);
            ok()
        })
    }
}
