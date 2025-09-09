use {
    crate::prelude::*,
    alloc::{
        boxed::Box,
        fmt::{Debug, Display},
        format,
        string::{String, ToString},
        sync::Arc,
        vec::Vec
    },
    core::ops::{Deref, DerefMut},
    serde::{Deserialize, Serialize}
};

pub trait AppConfigExt:
    Debug
    + Default
    + Send
    + Sync
    + 'static
    + for<'iter> Extend<(&'iter str, Option<&'iter str>)>
    + LoadArgs
    + LoadEnv
    + LoadDirs
{
    const CONFIG_FILE_NAME: &'static str = concat!(env!("APP_BIN"), ".ini");
    const DEFAULT_COMMAND: &'static str = "run";
}

#[derive(Debug, Default, ExtendFromIter, Serialize, Deserialize)]
pub struct AppConfig<C: AppConfigExt> {
    pub base: Arc<BaseConfig>,
    pub dirs: Arc<Dirs>,
    pub external: Arc<C>
}

impl<C> AppConfig<C>
where
    C: AppConfigExt
{
    pub const CONFIG_FILE_NAME: &'static str = C::CONFIG_FILE_NAME;
    pub const DEFAULT_COMMAND: &'static str = C::DEFAULT_COMMAND;

    pub fn load(&mut self, args: Option<&Args>) -> Ok<&mut Self> {
        let mut dirs = Dirs::default();
        dirs.load_env()?;

        if let Some(args) = args {
            dirs.load_args(args)?;
        }

        dirs.init();

        let config_file = format!("{}/{}", &dirs.config, Self::CONFIG_FILE_NAME);
        let mut ini = match Ini::from_file(&config_file) {
            Ok(ini) => {
                Env::is_debug().then(|| log::trace!("Loading: {config_file}"));
                ini
            },
            Err(e) => {
                match e.downcast_ref::<IniError>() {
                    Some(IniError::FileNotFound(..)) => Ini::default(),
                    _ => Err(e)?
                }
            },
        };

        let user_config_file = format!("{}/{}", &dirs.user_config, Self::CONFIG_FILE_NAME);
        match Ini::from_file(&user_config_file) {
            Ok(user_ini) => {
                Env::is_debug().then(|| log::trace!("Loading: {user_config_file}"));
                ini.extend(
                    user_ini
                        .into_iter()
                        .map(|(n, v)| (n.into(), v.map(|v| v.into())))
                );
            },
            Err(e) => {
                match e.downcast_ref::<IniError>() {
                    Some(IniError::FileNotFound(..)) => (),
                    _ => Err(e)?
                }
            },
        };

        self.extend(&ini);
        self.external.try_mut()?.extend(&ini);

        self.load_env()?;

        if let Some(args) = args {
            self.load_args(args)?;
        }

        self.dirs.try_mut()?.init();
        self.load_dirs(&self.dirs.clone())?;

        Ok(self)
    }
}

impl<C> AsRef<AppConfig<C>> for AppConfig<C>
where
    C: AppConfigExt
{
    fn as_ref(&self) -> &AppConfig<C> {
        self
    }
}

impl<C> AsMut<AppConfig<C>> for AppConfig<C>
where
    C: AppConfigExt
{
    fn as_mut(&mut self) -> &mut AppConfig<C> {
        self
    }
}

impl<C> Deref for AppConfig<C>
where
    C: AppConfigExt
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.external.as_ref()
    }
}

impl<C> DerefMut for AppConfig<C>
where
    C: AppConfigExt
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.external.try_mut().unwrap()
    }
}

impl<'a, C> From<&'a AppConfig<C>> for Vec<(&'static str, String)>
where
    C: AppConfigExt,
    &'a C: Into<Vec<(&'static str, String)>>
{
    fn from(value: &'a AppConfig<C>) -> Self {
        let env = Env::from_static();
        let mut res: Vec<_> = [
            ("env.env", &env.env as &dyn Display),
            ("env.is_prod", &env.is_prod),
            ("env.is_dev", &env.is_dev),
            ("env.is_debug", &env.is_debug),
            ("env.is_release", &env.is_release),
            ("base.language", &value.base.language),
            (
                "base.locales",
                Box::leak(Box::new(
                    value
                        .base
                        .locales
                        .iter()
                        .map(|(n, m)| format!("{n}={}", m.as_ref().unwrap_or(&"".into())))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            ),
            ("base.timezone", &value.base.timezone),
            ("base.log.level", &value.base.log.level),
            ("base.log.color", &value.base.log.color),
            (
                "base.log.filter",
                Box::leak(Box::new(
                    value
                        .base
                        .log
                        .filter
                        .as_ref()
                        .map(|v| v.join(","))
                        .unwrap_or_default()
                ))
            ),
            (
                "base.log.file",
                Box::leak(Box::new(value.base.log.file.as_deref().unwrap_or_default()))
            ),
            ("dirs.exe", Box::leak(Box::new(value.dirs.exe()))),
            ("dirs.bin", &value.dirs.bin),
            ("dirs.sbin", &value.dirs.sbin),
            ("dirs.lib", &value.dirs.lib),
            ("dirs.man", &value.dirs.man),
            ("dirs.doc", &value.dirs.doc),
            ("dirs.var", &value.dirs.var),
            ("dirs.run", &value.dirs.run),
            ("dirs.log", &value.dirs.log),
            ("dirs.data", &value.dirs.data),
            ("dirs.cache", &value.dirs.cache),
            ("dirs.state", &value.dirs.state),
            ("dirs.config", &value.dirs.config),
            ("dirs.user_config", &value.dirs.user_config),
            ("dirs.home", &value.dirs.home),
            ("dirs.include", &value.dirs.include),
            ("dirs.tmp", &value.dirs.tmp),
            ("dirs.prefix", &value.dirs.prefix),
            ("dirs.suffix", &value.dirs.suffix)
        ]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
        .collect();

        let custom: Vec<(&str, String)> = value.external.as_ref().into();
        res.extend(custom);

        res
    }
}

impl<'a, C> From<&'a mut AppConfig<C>> for Vec<&'a mut dyn LoadDirs>
where
    C: AppConfigExt
{
    fn from(value: &'a mut AppConfig<C>) -> Self {
        alloc::vec![
            &mut value.base.try_mut().unwrap().log,
            value.external.try_mut().unwrap()
        ]
    }
}

impl<'a, C> From<&'a mut AppConfig<C>> for Vec<&'a mut dyn LoadEnv>
where
    C: AppConfigExt
{
    fn from(value: &'a mut AppConfig<C>) -> Self {
        alloc::vec![
            value.base.try_mut().unwrap(),
            value.dirs.try_mut().unwrap(),
            value.external.try_mut().unwrap()
        ]
    }
}

impl<'a, C> From<&'a mut AppConfig<C>> for Vec<&'a mut dyn LoadArgs>
where
    C: AppConfigExt
{
    fn from(value: &'a mut AppConfig<C>) -> Self {
        alloc::vec![
            value.base.try_mut().unwrap(),
            value.dirs.try_mut().unwrap(),
            value.external.try_mut().unwrap()
        ]
    }
}

impl<C> LoadDirs for AppConfig<C>
where
    C: AppConfigExt
{
    fn load_dirs(&mut self, dirs: &Dirs) -> Void {
        let list = <Vec<&mut dyn LoadDirs>>::from(self);

        for config in list {
            config.load_dirs(dirs)?;
        }

        ok()
    }
}

impl<C> LoadEnv for AppConfig<C>
where
    C: AppConfigExt
{
    fn load_env(&mut self) -> Void {
        let list = <Vec<&mut dyn LoadEnv>>::from(self);

        for config in list {
            config.load_env()?;
        }

        ok()
    }
}

impl<C> LoadArgs for AppConfig<C>
where
    C: AppConfigExt
{
    fn load_args(&mut self, args: &Args) -> Void {
        let list: Vec<&mut dyn LoadArgs> = self.into();

        for config in list {
            config.load_args(args)?;
        }

        ok()
    }
}
