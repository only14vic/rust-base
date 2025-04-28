use {
    app_async::actix_on_tokio_start,
    app_base::{log_init, mem_stats, ok, Ini, Void}
};

mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/config.rs"
    ));
}

fn main() -> Void {
    Ini::dotenv(false).ok();
    log_init();
    let config = tests::Config::load()?;

    let res = actix_on_tokio_start((&config.tokio).into(), async {
        "Hello, from Async!"
    })?;
    println!("{res}");
    assert_eq!(res, "Hello, from Async!");

    mem_stats();

    ok()
}
