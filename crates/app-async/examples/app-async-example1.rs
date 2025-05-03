use {
    app_async::{actix_on_tokio_start, db::db_pool},
    app_base::prelude::*,
    sqlx::{Acquire, Postgres, Row}
};

mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/config.rs"
    ));
}

fn main() -> Void {
    dotenv(false);
    let mut log = Logger::init()?;

    let config = tests::Config::load()?;

    log.configure(&config.base.log)?;

    let res = actix_on_tokio_start((&config.tokio).into(), async {
        let db = db_pool::<Postgres>(Some(&config.db)).await?;
        let mut tasks = Vec::new();

        for _ in 0..100 {
            let db = db.clone();
            let task = tokio::spawn(async move {
                let mut conn = db.acquire().await?;
                for _ in 0..2000 {
                    let row = sqlx::query("select $1 as data")
                        .bind("Hello SQL!")
                        .fetch_one(conn.acquire().await?)
                        .await?;

                    let data = row.try_get::<String, &str>("data")?;
                    assert_eq!("Hello SQL!", &data);
                }

                ok() as Result<(), ErrAsync>
            });

            tasks.push(task);
        }

        futures::future::join_all(tasks)
            .await
            .into_iter()
            .for_each(|r| {
                let _ = r.unwrap().unwrap();
            });

        Ok("Hello, from Async!") as Result<&str, ErrAsync>
    })?;
    assert_eq!(res.unwrap(), "Hello, from Async!");

    mem_stats();

    ok()
}
