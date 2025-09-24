use {
    crate::MainModule,
    alloc::{fmt::Debug, format, string::String, sync::Arc, vec::Vec},
    app_async::{TokioConfig, db::DbConfig},
    app_base::prelude::*,
    serde::{Deserialize, Serialize}
};
#[cfg(feature = "migrator")]
use app_migrator::{MigratorConfig, MigratorConfigExt};
#[cfg(feature = "desktop")]
use app_desktop::{DesktopConfig, DesktopConfigExt};
#[cfg(any(feature = "web", feature = "desktop"))]
use app_web::{ActixConfig, WebConfig, WebConfigExt};

#[derive(Debug, Default, ExtendFromIter, Serialize, Deserialize)]
pub struct Config {
    pub tokio: Arc<TokioConfig>,
    pub db: Arc<DbConfig>,
    #[cfg(any(feature = "web", feature = "desktop"))]
    pub actix: Arc<ActixConfig>,
    #[cfg(any(feature = "web", feature = "desktop"))]
    pub web: Arc<WebConfig>,
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

#[cfg(any(feature = "web", feature = "desktop"))]
impl WebConfigExt for Config {}

#[cfg(feature = "migrator")]
impl MigratorConfigExt for Config {}

#[cfg(feature = "desktop")]
impl DesktopConfigExt for Config {}

impl Iter<'_, (&'static str, String)> for Config {
    fn iter(&'_ self) -> impl Iterator<Item = (&'static str, String)> {
        let mut res = Vec::new();
        res.extend(self.db.iter());
        res.extend(self.tokio.iter());
        #[cfg(any(feature = "web", feature = "desktop"))]
        res.extend(self.actix.iter());
        #[cfg(any(feature = "web", feature = "desktop"))]
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
            self.db.try_mut().unwrap() as &mut dyn LoadArgs,
            self.tokio.try_mut().unwrap(),
            #[cfg(any(feature = "web", feature = "desktop"))]
            self.actix.try_mut().unwrap(),
            #[cfg(any(feature = "web", feature = "desktop"))]
            self.web.try_mut().unwrap(),
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
            self.db.try_mut().unwrap() as &mut dyn LoadArgs,
            self.tokio.try_mut().unwrap(),
            #[cfg(any(feature = "web", feature = "desktop"))]
            self.actix.try_mut().unwrap(),
            #[cfg(any(feature = "web", feature = "desktop"))]
            self.web.try_mut().unwrap(),
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
            self.db.try_mut().unwrap() as &mut dyn LoadDirs,
            self.tokio.try_mut().unwrap(),
            #[cfg(any(feature = "web", feature = "desktop"))]
            self.actix.try_mut().unwrap(),
            #[cfg(any(feature = "web", feature = "desktop"))]
            self.web.try_mut().unwrap(),
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
            self.db.try_mut().unwrap() as &mut dyn LoadEnv,
            self.tokio.try_mut().unwrap(),
            #[cfg(any(feature = "web", feature = "desktop"))]
            self.actix.try_mut().unwrap(),
            #[cfg(any(feature = "web", feature = "desktop"))]
            self.web.try_mut().unwrap(),
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

impl AsRef<Arc<DbConfig>> for Config {
    #[inline]
    fn as_ref(&self) -> &Arc<DbConfig> {
        &self.db
    }
}

impl AsRef<Arc<TokioConfig>> for Config {
    #[inline]
    fn as_ref(&self) -> &Arc<TokioConfig> {
        &self.tokio
    }
}

#[cfg(any(feature = "web", feature = "desktop"))]
impl AsRef<Arc<ActixConfig>> for Config {
    #[inline]
    fn as_ref(&self) -> &Arc<ActixConfig> {
        &self.actix
    }
}

#[cfg(any(feature = "web", feature = "desktop"))]
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

#[cfg(feature = "desktop")]
impl AsMut<Arc<DesktopConfig>> for Config {
    #[inline]
    fn as_mut(&mut self) -> &mut Arc<DesktopConfig> {
        &mut self.desktop
    }
}
