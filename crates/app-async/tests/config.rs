use {
    app_async::{db::DbConfig, TokioConfig},
    app_base::prelude::*,
    std::env::current_dir
};

#[derive(Debug, Default, SetFromIter)]
pub struct Config {
    pub base: BaseConfig,
    pub tokio: TokioConfig,
    #[cfg(feature = "db")]
    pub db: DbConfig
}

impl Config {
    pub fn load() -> Ok<Self> {
        let mut config = Self::default();

        let mut file = current_dir()?;
        file.push("config/app.ini");
        let ini = Ini::from_file(&file.to_string_lossy())?;
        config.set_from_iter(&ini)?;

        config.load_env()?;

        let args = Args::new([
            ("tokio-threads", vec!["-t"], None),
            ("db-url", vec![], None),
            ("log-level", vec!["-l"], None),
            ("log-file", vec!["-f"], None)
        ])
        .parse_args(std::env::args().collect())?;
        config.load_args(&args)?;

        log::trace!("Loaded: {config:#?}");

        config.into_ok()
    }
}

impl LoadEnv for Config {
    fn load_env(&mut self) -> Ok<&mut Self> {
        self.base.load_env()?;
        self.tokio.load_env()?;
        #[cfg(feature = "db")]
        self.db.load_env()?;
        self.into_ok()
    }
}

impl LoadArgs for Config {
    fn load_args(&mut self, args: &Args) -> Ok<&mut Self> {
        self.base.log.load_args(&args)?;
        #[cfg(feature = "db")]
        self.db.load_args(&args)?;
        self.tokio.load_args(&args)?;
        self.into_ok()
    }
}
