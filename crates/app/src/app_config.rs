#[cfg(not(feature = "std"))]
use {core::ffi::c_char, core::ffi::c_int};
#[cfg(feature = "std")]
use {app_async::TokioConfig, app_web::ActixConfig};
#[cfg(feature = "db")]
use app_async::db::DbConfig;
use {
    crate::AppOptions,
    alloc::{
        boxed::Box,
        fmt::{Debug, Display},
        format,
        string::{String, ToString},
        sync::Arc
    },
    app_base::prelude::*,
    core::any::type_name
};

#[derive(Debug, Default, Extend)]
pub struct AppConfig {
    pub options: AppOptions,
    pub base: BaseConfig,
    pub dirs: Dirs,
    #[cfg(feature = "std")]
    pub tokio: TokioConfig,
    #[cfg(feature = "std")]
    pub actix: Arc<ActixConfig>,
    #[cfg(feature = "db")]
    pub db: Arc<DbConfig>
}

impl AppConfig {
    pub const CONFIG_FILE_NAME: &'static str = "app.ini";

    pub fn load(args: Option<&Args<'_>>) -> Ok<Self> {
        let mut dirs = Dirs::default();
        dirs.load_env()?;

        if let Some(args) = args {
            dirs.load_args(args)?;
        }

        dirs.init();

        let config_file = format!("{}/{}", &dirs.config, Self::CONFIG_FILE_NAME);
        let ini = Ini::from_file(&config_file)?;

        let mut config = Self::default();
        config.extend(&ini);
        config.load_env()?;

        if let Some(args) = args {
            config.load_args(args)?;
        }

        config.dirs.init();
        config.base.log.with_log_dir(&config.dirs.log);

        #[cfg(feature = "std")]
        Self::try_mut(&mut config.actix)?.with_dirs(&config.dirs);

        Ok(config)
    }

    pub fn parse_args(
        #[cfg(not(feature = "std"))] argc: c_int,
        #[cfg(not(feature = "std"))] argv: *const *const c_char
    ) -> Ok<Args<'static>> {
        let mut args = Args::new([
            ("exe", &["0"][..], None),
            ("command", &["1"][..], None),
            ("value", &["2"], None),
            ("log-level", &["-l"], None),
            ("log-color", &[], None),
            ("log-file", &[], None),
            ("log-filter", &[], None),
            ("language", &[], None),
            ("timezone", &[], None),
            ("home-dir", &["-h"], None),
            ("config-dir", &["-c"], None),
            ("user-config-dir", &["-u"], None),
            ("log-dir", &[], None),
            ("tokio-threads", &[], None),
            ("actix-threads", &[], None),
            ("actix-port", &[], None),
            ("db-url", &[], None)
        ])?;

        if Env::is_test() {
            args.allow_undefined(true);
        }

        #[cfg(feature = "std")]
        let args = args.parse_args(std::env::args().collect())?;
        #[cfg(not(feature = "std"))]
        let args = unsafe { args.parse_argc(argc, argv)? };

        Ok(args)
    }

    pub fn try_mut<T>(value: &mut Arc<T>) -> Ok<&mut T> {
        Arc::get_mut(value).ok_or(
            format!(
                "Could not get mutable reference of Arc<{}>",
                type_name::<T>()
            )
            .into()
        )
    }

    /// Creates iterator of string options
    ///
    /// May cause a memory leak!
    pub fn iter(&self) -> impl Iterator<Item = (&str, String)> {
        [
            &[
                (
                    "options.clear_static_di",
                    &self.options.clear_static_di as &(dyn Display + Send + Sync)
                ),
                ("base.language", &self.base.language),
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
            ] as &[(&str, &(dyn Display + Send + Sync))],
            #[cfg(feature = "std")]
            &[
                ("tokio.threads", &self.tokio.threads),
                ("tokio.blocking_threads", &self.tokio.blocking_threads),
                ("tokio.thread_name", &self.tokio.thread_name),
                ("actix.port", &self.actix.port),
                ("actix.socket", &self.actix.socket),
                ("actix.listen", &self.actix.listen),
                ("actix.threads", &self.actix.threads),
                (
                    "actix.blocking_threads_per_worker", &self.actix.blocking_threads_per_worker
                ),
                ("actix.static_dir", &self.actix.static_dir),
                ("actix.static_path", &self.actix.static_path)
            ],
            #[cfg(feature = "db")]
            &[
                ("db.url", &self.db.url),
                (
                    "db.schema",
                    Box::leak(Box::new(self.db.schema.as_deref().unwrap_or_default()))
                ),
                ("db.min_conn", &self.db.min_conn),
                ("db.max_conn", &self.db.max_conn),
                ("db.idle_timeout", &self.db.idle_timeout),
                ("db.max_lifetime", &self.db.max_lifetime),
                ("db.acquire_timeout", &self.db.acquire_timeout)
            ]
        ]
        .concat()
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
    }
}

impl LoadEnv for AppConfig {
    fn load_env(&mut self) -> Void {
        let list = [
            &mut self.base as &mut dyn LoadEnv,
            &mut self.dirs,
            #[cfg(feature = "std")]
            &mut self.tokio,
            #[cfg(feature = "std")]
            Self::try_mut(&mut self.actix)?,
            #[cfg(feature = "db")]
            Self::try_mut(&mut self.db)?
        ];

        for config in list {
            config.load_env()?;
        }

        ok()
    }
}

impl LoadArgs for AppConfig {
    fn load_args(&mut self, args: &Args) -> Void {
        let list = [
            &mut self.base as &mut dyn LoadArgs,
            &mut self.dirs,
            #[cfg(feature = "std")]
            &mut self.tokio,
            #[cfg(feature = "std")]
            Self::try_mut(&mut self.actix)?,
            #[cfg(feature = "db")]
            Self::try_mut(&mut self.db)?
        ];

        for config in list {
            config.load_args(args)?;
        }

        ok()
    }
}
