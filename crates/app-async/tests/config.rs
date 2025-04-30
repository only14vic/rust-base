use {app_async::TokioConfig, app_base::prelude::*};

#[derive(Debug, Default, SetFromIter)]
pub struct Config {
    pub base: BaseConfig,
    pub tokio: TokioConfig
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

    pub fn load_env(&mut self) -> Void {
        self.base.load_env()?;
        self.tokio.load_env()?;

        ok()
    }
}
