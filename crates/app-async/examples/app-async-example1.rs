use {
    app_async::actix_on_tokio_start,
    app_base::{log_init, ok, Ini, Void}
};

fn main() -> Void {
    Ini::dotenv(false)?;
    log_init();

    let res = actix_on_tokio_start(None, async { "Hello, from Async!" })?;

    println!("{res}");
    assert_eq!(res, "Hello, from Async!");

    ok()
}
