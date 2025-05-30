use {
    app_async::{db::DbConfig, http_server::ActixConfig, TokioConfig},
    app_base::prelude::*,
    std::env::current_dir
};

#[derive(Debug, Default, Extend)]
pub struct Config {
    pub base: BaseConfig,
    pub tokio: TokioConfig,
    #[cfg(feature = "db")]
    pub db: DbConfig,
    pub actix: ActixConfig
}

impl Config {
    pub fn load() -> Ok<Self> {
        let mut config = Self::default();

        let mut file = current_dir()?;
        file.push("config/app.ini");
        let ini = Ini::from_file(&file.to_string_lossy())?;
        config.extend(&ini);

        config.load_env()?;

        let args = Args::new([
            ("tokio-threads", &["-t"][..], None),
            #[cfg(feature = "db")]
            ("db-url", &[], None),
            ("log-level", &["-l"], None),
            ("log-file", &["-f"], None)
        ])
        .parse_args(std::env::args().collect())?;
        config.load_args(&args)?;

        log::trace!("Loaded: {config:#?}");

        config.into_ok()
    }
}

impl LoadEnv for Config {
    fn load_env(&mut self) -> Ok<()> {
        self.base.load_env()?;
        self.tokio.load_env()?;
        self.actix.load_env()?;
        #[cfg(feature = "db")]
        self.db.load_env()?;
        ok()
    }
}

impl LoadArgs for Config {
    fn load_args(&mut self, args: &Args) -> Ok<()> {
        self.base.log.load_args(&args)?;
        self.tokio.load_args(&args)?;
        self.actix.load_args(&args)?;
        #[cfg(feature = "db")]
        self.db.load_args(&args)?;
        ok()
    }
}
