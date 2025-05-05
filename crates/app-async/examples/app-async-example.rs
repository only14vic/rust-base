use {
    app_async::{
        actix_with_tokio_start,
        cache::{ArrayCache, Cache, Cacher},
        db::db_pool
    },
    app_base::prelude::*,
    core::sync::atomic::{AtomicU64, Ordering},
    sqlx::{Acquire, Postgres, Row},
    std::sync::Arc
};

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/common/config.rs"
));

const MAX_TASKS: usize = 10_000;
const MAX_ITERS: usize = 1_000;

fn main() -> Void {
    App::new([run, MODULE_APP_CONFIG]).boot()?.run()
}

fn run(app: &mut App, event: AppEvent) -> Void {
    if event == AppEvent::APP_INIT {
        app.register_command("run", run);
    }

    if event == AppEvent::APP_RUN {
        let config = app.config();

        let res = actix_with_tokio_start(Some(&config.tokio), async {
            let db = db_pool::<Postgres>(Some(&config.db)).await?;
            let mut tasks = Vec::new();
            let cache = Cacher::<ArrayCache>::from_static();

            log::info!("MAX iters: {}", MAX_TASKS * MAX_ITERS);

            let sql_counter = Arc::new(AtomicU64::new(0));

            for j in 0..MAX_TASKS {
                let db = db.clone();
                let cache = cache.clone();
                let sql_cn = sql_counter.clone();

                let task = tokio::task::spawn(async move {
                    let mut conn = db.acquire().await?;
                    let mut buff_j = itoa::Buffer::new();

                    for _i in 0..MAX_ITERS {
                        let keys = ["example", buff_j.format(j)];

                        cache
                            .getset(&keys, 10, async {
                                sql_cn.fetch_add(1, Ordering::Relaxed);

                                let row = sqlx::query("select now(), $1 as data")
                                    .bind("Hello SQL!")
                                    .fetch_one(conn.acquire().await?)
                                    .await?;

                                let data = row.try_get::<String, &str>("data")?;

                                Ok(data)
                            })
                            .await?;

                        assert_eq!(
                            "Hello SQL!",
                            cache.get::<String>(&keys).await?.unwrap().as_str()
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
                    r.unwrap().unwrap();
                });

            cache.clear_all().await?;

            log::info!("Executed SQL: {}", sql_counter.load(Ordering::Relaxed));

            Ok("Hello, from Async!") as Result<&str, ErrAsync>
        })?;
        assert_eq!(res.unwrap(), "Hello, from Async!");

        mem_stats();
    }

    ok()
}
