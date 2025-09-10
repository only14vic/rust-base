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
    Default
    + Send
    + Sync
    + 'static
    + Debug
    + for<'a> Extend<(&'a str, Option<&'a str>)>
    + for<'a> Iter<'a, (&'static str, String)>
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

impl<C> Iter<'_, (&'static str, String)> for AppConfig<C>
where
    C: AppConfigExt
{
    fn iter(&self) -> impl Iterator<Item = (&'static str, String)> {
        let env = Env::from_static();
        let mut res: Vec<_> = [
            ("env.env", &env.env as &dyn Display),
            ("env.is_prod", &env.is_prod),
            ("env.is_dev", &env.is_dev),
            ("env.is_debug", &env.is_debug),
            ("env.is_release", &env.is_release),
            ("base.language", &self.base.language),
            (
                "base.locales",
                Box::leak(Box::new(
                    self.base
                        .locales
                        .iter()
                        .map(|(n, m)| format!("{n}={}", m.as_ref().unwrap_or(&"".into())))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            ),
            ("base.timezone", &self.base.timezone),
            ("base.log.level", &self.base.log.level),
            ("base.log.color", &self.base.log.color),
            (
                "base.log.filter",
                Box::leak(Box::new(
                    self.base
                        .log
                        .filter
                        .as_ref()
                        .map(|v| v.join(","))
                        .unwrap_or_default()
                ))
            ),
            (
                "base.log.file",
                Box::leak(Box::new(self.base.log.file.as_deref().unwrap_or_default()))
            ),
            ("dirs.exe", Box::leak(Box::new(self.dirs.exe()))),
            ("dirs.bin", &self.dirs.bin),
            ("dirs.sbin", &self.dirs.sbin),
            ("dirs.lib", &self.dirs.lib),
            ("dirs.man", &self.dirs.man),
            ("dirs.doc", &self.dirs.doc),
            ("dirs.var", &self.dirs.var),
            ("dirs.run", &self.dirs.run),
            ("dirs.log", &self.dirs.log),
            ("dirs.data", &self.dirs.data),
            ("dirs.cache", &self.dirs.cache),
            ("dirs.state", &self.dirs.state),
            ("dirs.config", &self.dirs.config),
            ("dirs.user_config", &self.dirs.user_config),
            ("dirs.home", &self.dirs.home),
            ("dirs.include", &self.dirs.include),
            ("dirs.tmp", &self.dirs.tmp),
            ("dirs.prefix", &self.dirs.prefix),
            ("dirs.suffix", &self.dirs.suffix)
        ]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
        .collect();

        res.extend(self.external.iter());

        res.into_iter()
    }
}

impl<'a, C> IterMut<'a, &'a mut dyn LoadDirs> for AppConfig<C>
where
    C: AppConfigExt
{
    fn iter_mut(&'a mut self) -> impl Iterator<Item = &'a mut dyn LoadDirs> {
        [
            &mut self.base.try_mut().unwrap().log as &mut dyn LoadDirs,
            self.external.try_mut().unwrap()
        ]
        .into_iter()
    }
}

impl<'a, C> IterMut<'a, &'a mut dyn LoadEnv> for AppConfig<C>
where
    C: AppConfigExt
{
    fn iter_mut(&'a mut self) -> impl Iterator<Item = &'a mut dyn LoadEnv> {
        [
            self.base.try_mut().unwrap() as &mut dyn LoadEnv,
            self.dirs.try_mut().unwrap(),
            self.external.try_mut().unwrap()
        ]
        .into_iter()
    }
}

impl<'a, C> IterMut<'a, &'a mut dyn LoadArgs> for AppConfig<C>
where
    C: AppConfigExt
{
    fn iter_mut(&'a mut self) -> impl Iterator<Item = &'a mut dyn LoadArgs> {
        [
            self.base.try_mut().unwrap() as &mut dyn LoadArgs,
            self.dirs.try_mut().unwrap(),
            self.external.try_mut().unwrap()
        ]
        .into_iter()
    }
}

impl<C> LoadDirs for AppConfig<C>
where
    C: AppConfigExt
{
    fn load_dirs(&mut self, dirs: &Dirs) -> Void {
        let list = <Self as IterMut<&mut dyn LoadDirs>>::iter_mut(self).collect::<Vec<_>>();

        for item in list {
            item.load_dirs(dirs)?;
        }

        ok()
    }
}

impl<C> LoadEnv for AppConfig<C>
where
    C: AppConfigExt
{
    fn load_env(&mut self) -> Void {
        let list = <Self as IterMut<&mut dyn LoadEnv>>::iter_mut(self);

        for item in list {
            item.load_env()?;
        }

        ok()
    }
}

impl<C> LoadArgs for AppConfig<C>
where
    C: AppConfigExt
{
    fn load_args(&mut self, args: &Args) -> Void {
        let list = <Self as IterMut<&mut dyn LoadArgs>>::iter_mut(self);

        for item in list {
            item.load_args(args)?;
        }

        ok()
    }
}
