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
            ("log-file", &["-f"], None),
            ("language", &[], None),
            ("timezone", &[], None)
        ])?
        .parse_args(std::env::args().collect())?;
        config.load_args(&args)?;

        log::trace!("Loaded: {config:#?}");

        config.into_ok()
    }
}

impl LoadEnv for Config {
    fn load_env(&mut self) -> Ok<()> {
        #[rustfmt::skip]
        let list = [
            &mut self.base,
            &mut self.tokio,
            &mut self.actix,
            #[cfg(feature = "db")]
            &mut self.db
        ] as [&mut dyn LoadEnv; 4];

        for config in list {
            config.load_env()?;
        }

        ok()
    }
}

impl LoadArgs for Config {
    fn load_args(&mut self, args: &Args) -> Ok<()> {
        #[rustfmt::skip]
        let list = [
            &mut self.base,
            &mut self.tokio,
            &mut self.actix,
            #[cfg(feature = "db")]
            &mut self.db
        ] as [&mut dyn LoadArgs; 4];

        for config in list {
            config.load_args(args)?;
        }

        ok()
    }
}
