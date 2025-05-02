use {
    app_async::{db::DbConfig, TokioConfig},
    app_base::prelude::*
};

#[derive(Debug, Default, SetFromIter)]
pub struct Config {
    pub base: BaseConfig,
    pub tokio: TokioConfig,
    #[cfg(feature = "db")]
    pub db: DbConfig
}

impl Config {
    pub fn from_file(file: &str) -> Ok<Self> {
        let mut config = Self::default();

        let ini = Ini::from_file(&file)?;
        config.set_from_iter(&ini)?;
        config.load_env()?;

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
