#[cfg(feature = "std")]
use {app_async::TokioConfig, app_web::ActixConfig, app_web::WebConfig};
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
    pub base: Arc<BaseConfig>,
    pub dirs: Dirs,
    #[cfg(feature = "std")]
    pub tokio: TokioConfig,
    #[cfg(feature = "std")]
    pub actix: Arc<ActixConfig>,
    #[cfg(feature = "std")]
    pub web: Arc<WebConfig>,
    #[cfg(feature = "db")]
    pub db: Arc<DbConfig>
}

impl AppConfig {
    pub const CONFIG_FILE_NAME: &'static str = concat!(env!("APP_BIN"), ".ini");
    pub const DEFAULT_COMMAND: &'static str = "run";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&mut self, args: Option<&Args>) -> Ok<&mut Self> {
        let mut dirs = Dirs::default();
        dirs.load_env()?;

        if let Some(args) = args {
            dirs.load_args(args)?;
        }

        dirs.init();

        let config_file = format!("{}/{}", &dirs.config, Self::CONFIG_FILE_NAME);
        match Ini::from_file(&config_file) {
            Ok(ini) => {
                log::trace!("Loading: {config_file}");
                self.extend(&ini);
            },
            Err(e) => {
                match e.downcast_ref::<IniError>() {
                    Some(IniError::FileNotFound(..)) => (),
                    _ => Err(e)?
                }
            },
        };

        let user_config_file = format!("{}/{}", &dirs.user_config, Self::CONFIG_FILE_NAME);
        match Ini::from_file(&user_config_file) {
            Ok(ini) => {
                log::trace!("Loading: {user_config_file}");
                self.extend(&ini);
            },
            Err(e) => {
                match e.downcast_ref::<IniError>() {
                    Some(IniError::FileNotFound(..)) => (),
                    _ => Err(e)?
                }
            },
        };

        self.load_env()?;

        if let Some(args) = args {
            self.load_args(args)?;
        }

        self.dirs.init();
        Self::try_mut(&mut self.base)?
            .log
            .with_log_dir(&self.dirs.log);

        #[cfg(feature = "std")]
        Self::try_mut(&mut self.actix)?.with_dirs(&self.dirs);
        #[cfg(feature = "std")]
        Self::try_mut(&mut self.web)?.with_dirs(&self.dirs);

        Ok(self)
    }

    pub fn args() -> Ok<Args> {
        let mut args = Args::new([
            ("exe", &["0"][..], None),
            ("command", &["1"], Some(Self::DEFAULT_COMMAND)),
            ("value", &["2"], None),
            ("help", &["-h"], None),
            ("log-level", &[], None),
            ("log-color", &[], None),
            ("log-file", &[], None),
            ("log-filter", &[], None),
            ("language", &[], None),
            ("timezone", &[], None),
            ("home-dir", &[], None),
            ("config-dir", &[], None),
            ("user-config-dir", &[], None),
            ("log-dir", &[], None),
            ("var-dir", &[], None),
            ("run-dir", &[], None),
            ("data-dir", &[], None),
            ("cache-dir", &[], None),
            ("state-dir", &[], None),
            ("tmp-dir", &[], None),
            ("tokio-threads", &[], None),
            ("actix-socket", &[], None),
            ("actix-listen", &[], None),
            ("actix-port", &[], None),
            ("actix-threads", &[], None),
            ("db-url", &[], None),
            ("web-host", &[], None),
            ("web-hostname", &[], None),
            ("web-url", &[], None),
            ("web-static-dir", &[], None),
            ("web-static-path", &[], None)
        ])?;

        if Env::is_test() {
            args.allow_undefined(true);
        }

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
                ("web.host", &self.web.host),
                ("web.hostname", &self.web.hostname),
                ("web.url", &self.web.url),
                (
                    "web.trusted_hosts",
                    Box::leak(Box::new(self.web.trusted_hosts.join(",")))
                ),
                (
                    "web.accept_hosts",
                    Box::leak(Box::new(self.web.accept_hosts.join(",")))
                ),
                ("web.static_dir", &self.web.static_dir),
                ("web.static_path", &self.web.static_path),
                ("web.api.url", &self.web.api.url),
                ("web.api.path", &self.web.api.path),
                ("web.api.proxy_url", &self.web.api.proxy_url),
                ("web.jwt.secret", &self.web.jwt.secret),
                ("web.jwt.issuer", &self.web.jwt.issuer),
                ("web.jwt.audience", &self.web.jwt.audience),
                (
                    "web.jwt.access_token_lifetime", &self.web.jwt.access_token_lifetime
                ),
                (
                    "web.jwt.refresh_token_lifetime", &self.web.jwt.refresh_token_lifetime
                ),
                ("web.firewall.fails_anon", &self.web.firewall.fails_anon),
                ("web.firewall.fails_user", &self.web.firewall.fails_user),
                ("web.firewall.fails_period", &self.web.firewall.fails_period),
                ("web.firewall.total_fails", &self.web.firewall.total_fails),
                ("web.firewall.total_period", &self.web.firewall.total_period)
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
            Self::try_mut(&mut self.base)? as &mut dyn LoadEnv,
            &mut self.dirs,
            #[cfg(feature = "std")]
            &mut self.tokio,
            #[cfg(feature = "std")]
            Self::try_mut(&mut self.actix)?,
            #[cfg(feature = "std")]
            Self::try_mut(&mut self.web)?,
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
            Self::try_mut(&mut self.base)? as &mut dyn LoadArgs,
            &mut self.dirs,
            #[cfg(feature = "std")]
            &mut self.tokio,
            #[cfg(feature = "std")]
            Self::try_mut(&mut self.actix)?,
            #[cfg(feature = "std")]
            Self::try_mut(&mut self.web)?,
            #[cfg(feature = "db")]
            Self::try_mut(&mut self.db)?
        ];

        for config in list {
            config.load_args(args)?;
        }

        ok()
    }
}
