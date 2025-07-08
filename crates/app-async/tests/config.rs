use {
    app_async::{db::DbConfig, http_server::ActixConfig, TokioConfig},
    app_base::prelude::*,
    std::{path::PathBuf, sync::Arc}
};

#[derive(Debug, Default, Extend)]
pub struct Config {
    pub base: BaseConfig,
    pub dirs: Dirs,
    pub tokio: TokioConfig,
    #[cfg(feature = "db")]
    pub db: Arc<DbConfig>,
    pub actix: ActixConfig
}

impl Config {
    const CONFIG_FILE_NAME: &str = "app.ini";

    pub fn load() -> Ok<Self> {
        let args = Args::new([
            ("tokio-threads", &["-t"][..], None),
            #[cfg(feature = "db")]
            ("db-url", &[], None),
            ("log-level", &["-l"], None),
            ("log-file", &[], None),
            ("language", &[], None),
            ("timezone", &[], None),
            ("home-dir", &["-h"], None),
            ("config-dir", &["-c"], None),
            ("user-config-dir", &["-u"], None)
        ])?
        .parse_args(std::env::args().collect())?;

        let mut dirs = Dirs::default();
        dirs.load_env()?;
        dirs.load_args(&args)?;

        let mut config_file = PathBuf::from(&dirs.config);
        config_file.push(Self::CONFIG_FILE_NAME);

        let ini = Ini::from_file(&config_file.to_string_lossy())?;

        let mut config = Self::default();
        config.extend(&ini);
        config.load_env()?;
        config.load_args(&args)?;

        config.base.log.with_log_dir(&config.dirs.log);

        log::trace!("Loaded: {config:#?}");

        config.into_ok()
    }
}

impl LoadEnv for Config {
    fn load_env(&mut self) -> Void {
        #[rustfmt::skip]
        let list = [
            &mut self.base as &mut dyn LoadEnv,
            &mut self.dirs,
            &mut self.tokio,
            &mut self.actix,
            #[cfg(feature = "db")]
            Arc::get_mut(&mut self.db).expect("Could not get mut ref of DbConfig")
        ];

        for config in list {
            config.load_env()?;
        }

        ok()
    }
}

impl LoadArgs for Config {
    fn load_args(&mut self, args: &Args) -> Void {
        #[rustfmt::skip]
        let list = [
            &mut self.base as &mut dyn LoadArgs,
            &mut self.dirs,
            &mut self.tokio,
            &mut self.actix,
            #[cfg(feature = "db")]
            Arc::get_mut(&mut self.db).expect("Could not get mut ref of DbConfig")
        ];

        for config in list {
            config.load_args(args)?;
        }

        ok()
    }
}
