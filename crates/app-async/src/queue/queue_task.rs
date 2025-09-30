use {
    app_base::prelude::*,
    serde_json::Value,
    sqlx::{
        Acquire, Postgres,
        pool::PoolConnection,
        prelude::FromRow,
        types::{
            Uuid,
            chrono::{DateTime, Local}
        }
    }
};

#[derive(Debug, FromRow)]
pub struct QueueTask {
    pub id: Uuid,
    pub name: String,
    pub params: Option<Value>,
    pub delay: i16,
    pub attempt: i16,
    pub updated_at: DateTime<Local>
}

impl QueueTask {
    pub async fn start_process(
        id: &Uuid,
        conn: &mut PoolConnection<Postgres>
    ) -> OkAsync<Option<Self>> {
        let task: Option<Self> =
            sqlx::query_as("select * from app.queue_start_process($1)")
                .bind(id)
                .fetch_optional(conn.acquire().await?)
                .await?;
        Ok(task)
    }

    pub async fn finish_process(
        self,
        error: Option<&str>,
        conn: &mut PoolConnection<Postgres>
    ) -> VoidAsync {
        sqlx::query("select * from app.queue_finish_process($1, $2)")
            .bind(self.id)
            .bind(error)
            .execute(conn.acquire().await?)
            .await?;
        ok()
    }
}
