use {
    crate::prelude::*,
    alloc::{
        boxed::Box,
        fmt::Debug,
        format,
        string::{String, ToString},
        sync::Arc,
        vec::Vec
    },
    core::{
        fmt::Display,
        ops::{Deref, DerefMut}
    },
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
    const CONFIG_FILE_NAME: &str = concat!(env!("APP_BIN"), ".ini");
    const DEFAULT_COMMAND: &str = "run";
}

#[derive(Debug, ExtendFromIter, Serialize, Deserialize)]
pub struct AppConfig<C: AppConfigExt> {
    pub name: Box<str>,
    pub bin: Box<str>,
    pub version: Box<str>,
    pub env_file: Option<Box<str>>,
    pub base: Arc<BaseConfig>,
    pub dirs: Arc<Dirs>,
    pub external: Arc<C>
}

impl<C> Default for AppConfig<C>
where
    C: AppConfigExt
{
    fn default() -> Self {
        Self {
            name: env!("APP_NAME").trim_matches(['\'', '"']).into(),
            bin: env!("APP_BIN").into(),
            version: concat!("v", env!("APP_VERSION"), " (", env!("BUILD_TIME"), ")")
                .into(),
            env_file: None,
            base: Default::default(),
            dirs: Default::default(),
            external: Default::default()
        }
    }
}

impl<C> AppConfigExt for AppConfig<C>
where
    C: AppConfigExt
{
    const DEFAULT_COMMAND: &str = "config";
    const CONFIG_FILE_NAME: &str = C::CONFIG_FILE_NAME;
}

impl<C> Deref for AppConfig<C>
where
    C: AppConfigExt
{
    type Target = C;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.external.as_ref()
    }
}

impl<C> DerefMut for AppConfig<C>
where
    C: AppConfigExt
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.external.try_mut().unwrap()
    }
}

impl<C> AsRef<Arc<BaseConfig>> for AppConfig<C>
where
    C: AppConfigExt
{
    #[inline]
    fn as_ref(&self) -> &Arc<BaseConfig> {
        &self.base
    }
}

impl<C> AsRef<Arc<Dirs>> for AppConfig<C>
where
    C: AppConfigExt
{
    #[inline]
    fn as_ref(&self) -> &Arc<Dirs> {
        &self.dirs
    }
}

impl<C> Iter<'_, (&'static str, String)> for AppConfig<C>
where
    C: AppConfigExt
{
    fn iter(&self) -> impl Iterator<Item = (&'static str, String)> {
        let mut res = Vec::new();

        res.extend(
            [
                // app
                ("app.name", &self.name as &dyn Display),
                ("app.bin", &self.bin),
                ("app.version", &self.version),
                (
                    "app.env_file",
                    &self.env_file.as_ref().map(|v| v.as_ref()).unwrap_or("")
                ),
                ("app.config_file_name", &Self::CONFIG_FILE_NAME),
                ("app.no_std", &cfg!(not(feature = "std"))),
                ("app.features", &env!("BUILD_FEATURES")),
                ("app.profile", &env!("BUILD_PROFILE")),
                ("app.default_command", &C::DEFAULT_COMMAND),
                // env
                ("env.env", &Env::env() as &dyn Display),
                ("env.is_prod", &Env::is_prod()),
                ("env.is_dev", &Env::is_dev()),
                ("env.is_test", &Env::is_test()),
                ("env.is_debug", &Env::is_debug()),
                ("env.is_release", &Env::is_release())
            ]
            .into_iter()
            .map(|(k, v)| (k, v.to_string()))
        );

        res.extend(self.base.iter());
        res.extend(self.dirs.iter());
        res.extend(self.external.iter());
        res.into_iter()
    }
}

impl<C> LoadArgs for AppConfig<C>
where
    C: AppConfigExt
{
    fn init_args(&mut self, args: &mut Args) {
        let list = [
            self.base.try_mut().unwrap() as &mut dyn LoadArgs,
            self.dirs.try_mut().unwrap(),
            self.external.try_mut().unwrap()
        ];

        for item in list {
            item.init_args(args);
        }
    }

    fn load_args(&mut self, args: &Args) {
        self.extend(
            [
                ("debug", args.get("debug")),
                ("env_file", args.get("env-file")),
                ("command", args.get("command"))
            ]
            .iter()
            .map(convert::tuple_result_option_str)
        );

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

impl<C> AppConfig<C>
where
    C: AppConfigExt
{
    #[inline]
    pub fn get<T>(&self) -> &Arc<T>
    where
        C: AsRef<Arc<T>>
    {
        (**self).as_ref()
    }

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
                Env::is_debug().then(|| log::debug!("Loading {config_file}"));
                ini
            },
            Err(e) => {
                match e.downcast_ref::<IniError>() {
                    Some(IniError::FileNotFound(..)) => Ini::default(),
                    _ => Err(e)?
                }
            },
        };

        let user_config_file =
            format!("{}/{}", &dirs.user_config, Self::CONFIG_FILE_NAME);
        match Ini::from_file(&user_config_file) {
            Ok(user_ini) => {
                Env::is_debug().then(|| log::debug!("Loading {user_config_file}"));
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
