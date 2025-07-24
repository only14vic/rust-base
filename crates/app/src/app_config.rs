#[cfg(not(feature = "std"))]
use {core::ffi::c_char, core::ffi::c_int};
#[cfg(feature = "std")]
use {app_async::TokioConfig, app_web::ActixConfig};
#[cfg(feature = "db")]
use app_async::db::DbConfig;
use {
    crate::AppOptions,
    alloc::{format, sync::Arc},
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
    pub fn load(config_file_name: &str, args: Option<&Args<'_>>) -> Ok<Self> {
        let mut dirs = Dirs::default();
        dirs.load_env()?;

        if let Some(args) = args {
            dirs.load_args(args)?;
        }

        dirs.init();

        let config_file = format!("{}/{config_file_name}", &dirs.config);
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
        Self::try_mut(&mut config.actix)?.with_socket_dir(&config.dirs.run);

        Ok(config)
    }

    pub fn parse_args(
        #[cfg(not(feature = "std"))] argc: c_int,
        #[cfg(not(feature = "std"))] argv: *const *const c_char
    ) -> Ok<Args<'static>> {
        let mut args = Args::new([
            ("log-level", &["-l"][..], None),
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
