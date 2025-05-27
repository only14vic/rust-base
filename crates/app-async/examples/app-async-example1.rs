use {
    app_async::{
        actix_on_tokio_start,
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

    let res = actix_on_tokio_start((&config.tokio).into(), async {
        let db = db_pool::<Postgres>(config.db.clone().into_some()).await?;
        let mut tasks = Vec::new();
        let cache = Cacher::<ArrayCache>::from_static();

        const MAX_TASKS: usize = 100;
        const MAX_ITERS: usize = 2000;

        for j in 0..MAX_TASKS {
            let db = db.clone();
            let cache = cache.clone();
            let task = tokio::spawn(async move {
                let mut conn = db.acquire().await?;
                for i in 0..MAX_ITERS {
                    let row = sqlx::query("select $1 as data")
                        .bind("Hello SQL!")
                        .fetch_one(conn.acquire().await?)
                        .await?;

                    let data = row.try_get::<String, &str>("data")?;

                    cache
                        .set(
                            "example",
                            &[&j.to_string(), &i.to_string()],
                            data.clone(),
                            1
                        )
                        .await?;

                    assert_eq!(
                        "Hello SQL!",
                        cache
                            .get::<String>("example", &[&j.to_string(), &i.to_string()])
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
