#[cfg(not(feature = "std"))]
use {core::ffi::c_char, core::ffi::c_int};
#[cfg(feature = "std")]
use {
    alloc::sync::Arc,
    app_async::{http_server::ActixConfig, TokioConfig}
};
#[cfg(feature = "db")]
use app_async::db::DbConfig;
use {crate::AppOptions, alloc::format, app_base::prelude::*};

#[derive(Debug, Default, Extend)]
pub struct AppConfig {
    pub options: AppOptions,
    pub base: BaseConfig,
    pub dirs: Dirs,
    #[cfg(feature = "std")]
    pub tokio: TokioConfig,
    #[cfg(feature = "std")]
    pub actix: ActixConfig,
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

        let config_file = format!("{}/{config_file_name}", &dirs.config);
        let ini = Ini::from_file(&config_file)?;

        let mut config = Self::default();
        config.extend(&ini);
        config.load_env()?;

        if let Some(args) = args {
            config.load_args(args)?;
        }

        config.base.log.with_log_dir(&config.dirs.log);

        config.into_ok()
    }

    pub fn parse_args(
        #[cfg(not(feature = "std"))] argc: c_int,
        #[cfg(not(feature = "std"))] argv: *const *const c_char
    ) -> Ok<Args<'static>> {
        let args = Args::new([
            ("log-level", &["-l"][..], None),
            ("log-file", &[], None),
            ("language", &[], None),
            ("timezone", &[], None),
            ("home-dir", &["-h"], None),
            ("config-dir", &["-c"], None),
            ("user-config-dir", &["-u"], None),
            ("log-dir", &[], None),
            ("tokio-threads", &["-t"], None),
            ("db-url", &[], None)
        ])?;

        #[cfg(feature = "std")]
        let args = args.parse_args(std::env::args().collect())?;
        #[cfg(not(feature = "std"))]
        let args = unsafe { args.parse_argc(argc, argv)? };

        Ok(args)
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
            &mut self.actix,
            #[cfg(feature = "db")]
            Arc::get_mut(&mut self.db).expect("Could not get mut ref of DbConfig")
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
            &mut self.actix,
            #[cfg(feature = "db")]
            Arc::get_mut(&mut self.db).expect("Could not get mut ref of DbConfig")
        ];

        for config in list {
            config.load_args(args)?;
        }

        ok()
    }
}
