use {
    app_async::TokioConfig,
    app_base::prelude::*,
    std::env::{self, current_dir}
};

#[derive(Debug, Default, SetFromIter)]
pub struct Config {
    pub tokio: TokioConfig
}

impl Config {
    pub fn load() -> Ok<Self> {
        let mut config = Self::default();

        let mut path = current_dir()?;
        path.push("config/app.ini");

        let ini = Ini::from_file(&path.to_string_lossy())?;
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

        log::trace!("Loaded: {config:#?}");

        config.into_ok()
    }
}
