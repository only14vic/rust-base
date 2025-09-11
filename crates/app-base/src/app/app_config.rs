use {
    crate::prelude::*,
    alloc::{fmt::Debug, format, string::String, sync::Arc, vec::Vec},
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
    + InitArgs
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
        dirs.load_env();

        if let Some(args) = args {
            dirs.load_args(args);
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

        self.load_env();

        if let Some(args) = args {
            self.load_args(args);
        }

        self.dirs.try_mut()?.init();
        self.load_dirs(&self.dirs.clone());

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
        let mut res = Vec::new();
        res.extend(self.base.iter());
        res.extend(self.dirs.iter());
        res.extend(self.external.iter());
        res.into_iter()
    }
}

impl<C> InitArgs for AppConfig<C>
where
    C: AppConfigExt
{
    fn init_args(&mut self, args: &mut Args) {
        let list = [
            self.base.try_mut().unwrap() as &mut dyn InitArgs,
            self.dirs.try_mut().unwrap(),
            self.external.try_mut().unwrap()
        ];

        for item in list {
            item.init_args(args);
        }
    }
}

impl<C> LoadArgs for AppConfig<C>
where
    C: AppConfigExt
{
    fn load_args(&mut self, args: &Args) {
        let list = [
            self.base.try_mut().unwrap() as &mut dyn LoadArgs,
            self.dirs.try_mut().unwrap(),
            self.external.try_mut().unwrap()
        ];

        for item in list {
            item.load_args(args);
        }
    }
}

impl<C> LoadDirs for AppConfig<C>
where
    C: AppConfigExt
{
    fn load_dirs(&mut self, dirs: &Dirs) {
        let list = [
            self.base.try_mut().unwrap() as &mut dyn LoadDirs,
            self.external.try_mut().unwrap()
        ];

        for item in list {
            item.load_dirs(dirs);
        }
    }
}

impl<C> LoadEnv for AppConfig<C>
where
    C: AppConfigExt
{
    fn load_env(&mut self) {
        let list = [
            self.base.try_mut().unwrap() as &mut dyn LoadEnv,
            self.dirs.try_mut().unwrap(),
            self.external.try_mut().unwrap()
        ];

        for item in list {
            item.load_env();
        }
    }
}
