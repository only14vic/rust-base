use {
    app_base::prelude::*,
    serde_json::Value,
    sqlx::{
        PgConnection,
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
        conn: &mut PgConnection
    ) -> OkAsync<Option<Self>> {
        let task: Option<Self> = sqlx::query_as(
            "select r.*
            from app.queue_start_process($1) as r
            where r.id is not null"
        )
        .bind(id)
        .fetch_optional(conn)
        .await?;
        Ok(task)
    }

    pub async fn finish_process(
        self,
        error: Option<&str>,
        conn: &mut PgConnection
    ) -> VoidAsync {
        sqlx::query("select app.queue_finish_process($1, $2)")
            .bind(self.id)
            .bind(error)
            .execute(conn)
            .await?;
        ok()
    }
}
