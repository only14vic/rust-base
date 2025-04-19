use {
    app_async::{actix_on_tokio_start, TokioConfig},
    app_base::{log_init, mem_stats, ok, BaseFromInto, Ini, Ok, SetFromIter, Void},
    std::env::{self, current_dir}
};

#[derive(Debug, Default, SetFromIter)]
struct Config {
    tokio: TokioConfig
}

impl Config {
    pub fn load() -> Ok<Self> {
        let mut path = current_dir()?;
        path.push("config/app.ini");

        let ini = Ini::from_file(&path.to_string_lossy())?;

        let mut config = Self::default();
        config.set_from_iter(&ini)?;

        let env_vars = [(
            "tokio.worker_threads",
            env::var("TOKIO_WORKER_THREADS").ok()
        )];
        config.set_from_iter(
            env_vars
                .iter()
                .map(|(k, v)| (*k, v.as_ref().map(String::as_str)))
        )?;

        config.into_ok()
    }
}

fn main() -> Void {
    Ini::dotenv(false).ok();
    log_init();

    let config = Config::load()?;
    let res = actix_on_tokio_start(Some(&config.tokio), async { "Hello, from Async!" })?;

    println!("{res}");
    assert_eq!(res, "Hello, from Async!");

    mem_stats();

    ok()
}
