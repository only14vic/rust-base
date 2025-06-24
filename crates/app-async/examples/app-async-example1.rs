use {
    app_async::{
        actix_with_tokio_start,
        cache::{ArrayCache, Cache, Cacher},
        db::db_pool
    },
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

    let res = actix_with_tokio_start(Some(&config.tokio), async {
        let db = db_pool::<Postgres>(Some(&config.db)).await?;
        let mut tasks = Vec::new();
        let cache = Cacher::<ArrayCache>::from_static();

        const MAX_TASKS: usize = 100;
        const MAX_ITERS: usize = 1000;

        for j in 0..MAX_TASKS {
            let db = db.clone();
            let cache = cache.clone();

            let task = tokio::task::spawn(async move {
                let mut conn = db.acquire().await?;
                let mut buff_j = itoa::Buffer::new();
                let mut buff_i = itoa::Buffer::new();

                for i in 0..MAX_ITERS {
                    let keys = [buff_j.format(j), buff_i.format(i)];

                    cache
                        .call("example", &keys, 2, async {
                            let row = sqlx::query("select $1 as data")
                                .bind("Hello SQL!")
                                .fetch_one(conn.acquire().await?)
                                .await?;

                            let data = row.try_get::<String, &str>("data")?;

                            Ok(data)
                        })
                        .await?;

                    assert_eq!(
                        "Hello SQL!",
                        cache
                            .get::<String>("example", &keys)
                            .await?
                            .unwrap()
                            .as_str()
                    );
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

        assert_eq!(MAX_TASKS * MAX_ITERS, cache.len().await);

        cache.clear_all().await?;

        Ok("Hello, from Async!") as Result<&str, ErrAsync>
    })?;
    assert_eq!(res.unwrap(), "Hello, from Async!");

    mem_stats();

    ok()
}
