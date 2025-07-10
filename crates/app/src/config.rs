#[cfg(not(feature = "std"))]
use core::ffi::c_char;

#[cfg(feature = "std")]
use app_async::{http_server::ActixConfig, TokioConfig};
#[cfg(feature = "db")]
use {alloc::sync::Arc, app_async::db::DbConfig};
use {
    alloc::{boxed::Box, format},
    app_base::prelude::*,
    core::pin::Pin
};

#[derive(Debug, Default, Extend)]
pub struct Config {
    pub base: BaseConfig,
    pub dirs: Dirs,
    #[cfg(feature = "std")]
    pub tokio: TokioConfig,
    #[cfg(feature = "std")]
    pub actix: ActixConfig,
    #[cfg(feature = "db")]
    pub db: Arc<DbConfig>
}

impl Config {
    pub fn load(
        config_file_name: &str,
        #[cfg(not(feature = "std"))] argc: usize,
        #[cfg(not(feature = "std"))] argv: *const *const c_char
    ) -> Ok<Self> {
        let mut args = Args::new([
            ("log-level", &["-l"][..], None),
            ("log-file", &[], None),
            ("language", &[], None),
            ("timezone", &[], None),
            ("home-dir", &["-h"], None),
            ("config-dir", &["-c"], None),
            ("user-config-dir", &["-u"], None),
            ("tokio-threads", &["-t"], None),
            ("db-url", &[], None)
        ])?;

        #[cfg(feature = "std")]
        {
            args = args.parse_args(std::env::args().collect())?;
        }
        #[cfg(not(feature = "std"))]
        {
            args = unsafe { args.parse_argc(argc, argv)? };
        }

        let mut dirs = Dirs::default();
        dirs.load_env()?;
        dirs.load_args(&args)?;

        let config_file = format!("{}/{config_file_name}", &dirs.config);
        let ini = Ini::from_file(&config_file)?;

        let mut config = Self::default();
        config.extend(&ini);
        config.load_env()?;
        config.load_args(&args)?;
        config.base.log.with_log_dir(&config.dirs.log);

        let di = Di::from_static();
        di.set(args);

        if let Ok(Some(log)) = di.get_mut::<Pin<Box<Logger>>>() {
            log.configure(&config.base.log)?;
        }

        log::trace!("Loaded: {config:#?}");

        config.into_ok()
    }
}

impl LoadEnv for Config {
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

impl LoadArgs for Config {
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
