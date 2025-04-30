use {app_async::actix_on_tokio_start, app_base::prelude::*, std::env::current_dir};

mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/config.rs"
    ));
}

fn main() -> Void {
    Ini::dotenv(false).ok();
    let mut log = Logger::init()?;

    let mut file = current_dir()?;
    file.push("config/app.ini");
    let config = tests::Config::from_file(&file.to_string_lossy())?;

    log.configure(&config.base.log)?;

    let res = actix_on_tokio_start((&config.tokio).into(), async {
        for _ in 0..5 {
            tokio::spawn(async {
                log::trace!(
                    "{:?} {:?}",
                    std::thread::current().name(),
                    std::thread::current().id()
                );
            });
        }

        tokio::task::yield_now().await;

        "Hello, from Async!"
    })?;
    println!("{res}");
    assert_eq!(res, "Hello, from Async!");

    mem_stats();

    ok()
}
