use {
    crate::MainModule,
    alloc::{fmt::Debug, format, string::String, sync::Arc, vec::Vec},
    app_async::{TokioConfig, db::DbConfig},
    app_base::prelude::*,
    app_web::{ActixConfig, WebConfig},
    serde::{Deserialize, Serialize}
};
#[cfg(feature = "migrator")]
use app_migrator::{MigratorConfig, MigratorConfigExt};
#[cfg(feature = "desktop")]
use app_desktop::{DesktopConfig, DesktopConfigExt};

#[derive(Debug, Default, ExtendFromIter, Serialize, Deserialize)]
pub struct Config {
    pub tokio: Arc<TokioConfig>,
    pub actix: Arc<ActixConfig>,
    pub web: Arc<WebConfig>,
    pub db: Arc<DbConfig>,
    #[cfg(feature = "migrator")]
    pub migrator: Arc<MigratorConfig>,
    #[cfg(feature = "desktop")]
    pub desktop: Arc<DesktopConfig>
}

impl AppConfigExt for Config {
    const COMMAND: &str = MainModule::COMMAND;
    const CONFIG_FILE_NAME: &str = concat!(env!("APP_BIN"), ".ini");
    const FEATURES: &str = env!("BUILD_FEATURES");
}

#[cfg(feature = "migrator")]
impl MigratorConfigExt for Config {}

#[cfg(feature = "desktop")]
impl DesktopConfigExt for Config {}

impl Iter<'_, (&'static str, String)> for Config {
    fn iter(&'_ self) -> impl Iterator<Item = (&'static str, String)> {
        let mut res = Vec::new();
        res.extend(self.db.iter());
        res.extend(self.tokio.iter());
        res.extend(self.actix.iter());
        res.extend(self.web.iter());
        #[cfg(feature = "migrator")]
        res.extend(self.migrator.iter());
        #[cfg(feature = "desktop")]
        res.extend(self.desktop.iter());
        res.into_iter()
    }
}

impl LoadArgs for Config {
    fn init_args(&mut self, args: &mut Args) {
        let list = [
            self.tokio.try_mut().unwrap() as &mut dyn LoadArgs,
            self.actix.try_mut().unwrap(),
            self.web.try_mut().unwrap(),
            self.db.try_mut().unwrap(),
            #[cfg(feature = "migrator")]
            self.migrator.try_mut().unwrap(),
            #[cfg(feature = "desktop")]
            self.desktop.try_mut().unwrap()
        ];

        for item in list {
            item.init_args(args);
        }
    }

    fn load_args(&mut self, args: &Args) {
        let list = [
            self.tokio.try_mut().unwrap() as &mut dyn LoadArgs,
            self.actix.try_mut().unwrap(),
            self.web.try_mut().unwrap(),
            self.db.try_mut().unwrap(),
            #[cfg(feature = "migrator")]
            self.migrator.try_mut().unwrap(),
            #[cfg(feature = "desktop")]
            self.desktop.try_mut().unwrap()
        ];

        for item in list {
            item.load_args(args);
        }
    }
}

impl LoadDirs for Config {
    fn load_dirs(&mut self, dirs: &Dirs) {
        let list = [
            self.tokio.try_mut().unwrap() as &mut dyn LoadDirs,
            self.actix.try_mut().unwrap(),
            self.web.try_mut().unwrap(),
            self.db.try_mut().unwrap(),
            #[cfg(feature = "migrator")]
            self.migrator.try_mut().unwrap(),
            #[cfg(feature = "desktop")]
            self.desktop.try_mut().unwrap()
        ];

        for item in list {
            item.load_dirs(dirs);
        }
    }
}

impl LoadEnv for Config {
    fn load_env(&mut self) {
        let list = [
            self.tokio.try_mut().unwrap() as &mut dyn LoadEnv,
            self.actix.try_mut().unwrap(),
            self.web.try_mut().unwrap(),
            self.db.try_mut().unwrap(),
            #[cfg(feature = "migrator")]
            self.migrator.try_mut().unwrap(),
            #[cfg(feature = "desktop")]
            self.desktop.try_mut().unwrap()
        ];

        for item in list {
            item.load_env();
        }
    }
}

impl AsRef<Arc<TokioConfig>> for Config {
    #[inline]
    fn as_ref(&self) -> &Arc<TokioConfig> {
        &self.tokio
    }
}

impl AsRef<Arc<ActixConfig>> for Config {
    #[inline]
    fn as_ref(&self) -> &Arc<ActixConfig> {
        &self.actix
    }
}

impl AsRef<Arc<DbConfig>> for Config {
    #[inline]
    fn as_ref(&self) -> &Arc<DbConfig> {
        &self.db
    }
}

impl AsRef<Arc<WebConfig>> for Config {
    #[inline]
    fn as_ref(&self) -> &Arc<WebConfig> {
        &self.web
    }
}

#[cfg(feature = "migrator")]
impl AsRef<Arc<MigratorConfig>> for Config {
    #[inline]
    fn as_ref(&self) -> &Arc<MigratorConfig> {
        &self.migrator
    }
}

#[cfg(feature = "desktop")]
impl AsRef<Arc<DesktopConfig>> for Config {
    #[inline]
    fn as_ref(&self) -> &Arc<DesktopConfig> {
        &self.desktop
    }
}
