use {
    app_async::{TokioConfig, db::DbConfig},
    serde::{Deserialize, Serialize},
    std::{fmt::Debug, format, string::String, vec::Vec}
};

pub type App = app_base::prelude::App<Config>;
pub type AppConfig = app_base::prelude::AppConfig<Config>;

pub const MODULE_APP_CONFIG: AppModule<Config> = AppConfigModule::<Config>::handle;

#[derive(Debug, Default, ExtendFromIter, Serialize, Deserialize)]
pub struct Config {
    pub tokio: Arc<TokioConfig>,
    pub db: Arc<DbConfig>
}

impl AppConfigExt for Config {
    const DEFAULT_COMMAND: &str = "run";
}

impl Iter<'_, (&'static str, String)> for Config {
    fn iter(&'_ self) -> impl Iterator<Item = (&'static str, String)> {
        [].into_iter()
    }
}

impl LoadArgs for Config {
    fn init_args(&mut self, args: &mut Args) {
        let list = [
            self.tokio.try_mut().unwrap() as &mut dyn LoadArgs,
            self.db.try_mut().unwrap()
        ];

        for item in list {
            item.init_args(args);
        }
    }

    fn load_args(&mut self, args: &Args) {
        let list = [
            self.tokio.try_mut().unwrap() as &mut dyn LoadArgs,
            self.db.try_mut().unwrap()
        ];

        for item in list {
            item.load_args(args);
        }
    }
}

impl LoadDirs for Config {
    fn load_dirs(&mut self, dirs: &Dirs) {
        let list = [] as [&mut dyn LoadDirs; 0];

        for item in list {
            item.load_dirs(dirs);
        }
    }
}

impl LoadEnv for Config {
    fn load_env(&mut self) {
        let list = [
            self.tokio.try_mut().unwrap() as &mut dyn LoadEnv,
            self.db.try_mut().unwrap()
        ];

        for item in list {
            item.load_env();
        }
    }
}
