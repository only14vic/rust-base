use {
    crate::{
        ActixConfig, HtmlRenderConfig,
        api::ApiConfig,
        ext::{AuthConfig, FirewallConfig, JwtConfig}
    },
    app_async::{TokioConfig, db::DbConfig},
    app_base::prelude::*,
    core::{fmt::Display, num::NonZero},
    serde::{Deserialize, Serialize},
    std::sync::Arc
};

pub trait WebConfigExt:
    AppConfigExt
    + AsRef<Arc<TokioConfig>>
    + AsRef<Arc<ActixConfig>>
    + AsRef<Arc<WebConfig>>
    + AsRef<Arc<DbConfig>>
{
}

#[derive(Debug, ExtendFromIter, Serialize, Deserialize)]
pub struct WebConfig {
    pub host: String,
    pub hostname: String,
    pub base_url: String,
    pub trusted_hosts: Vec<String>,
    pub accept_hosts: Vec<String>,
    pub static_path: String,
    pub static_dir: String,
    pub static_cache: Option<NonZero<u32>>,
    pub api: ApiConfig,
    pub jwt: JwtConfig,
    pub auth: AuthConfig,
    pub firewall: FirewallConfig,
    pub html_render: Arc<HtmlRenderConfig>
}

impl AppConfigExt for WebConfig {
    const COMMAND: &str = "serve";
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            host: "localhost:80".into(),
            hostname: "localhost".into(),
            base_url: "http://localhost:80".into(),
            trusted_hosts: vec![],
            accept_hosts: vec![],
            static_path: "/public".into(),
            static_dir: "public".into(),
            static_cache: NonZero::new(300),
            api: Default::default(),
            jwt: Default::default(),
            auth: Default::default(),
            firewall: Default::default(),
            html_render: Default::default()
        }
    }
}

impl Iter<'_, (&'static str, String)> for WebConfig {
    fn iter(&self) -> impl Iterator<Item = (&'static str, String)> {
        let mut res: Vec<_> = [
            ("web.host", &self.host as &dyn Display),
            ("web.hostname", &self.hostname),
            ("web.base_url", &self.base_url),
            (
                "web.trusted_hosts",
                Box::leak(Box::new(self.trusted_hosts.join(",")))
            ),
            (
                "web.accept_hosts",
                Box::leak(Box::new(self.accept_hosts.join(",")))
            ),
            ("web.static_dir", &self.static_dir),
            ("web.static_path", &self.static_path),
            (
                "web.static_cache",
                Box::leak(Box::new(
                    self.static_cache.map(|v| v.get()).unwrap_or_default()
                ))
            )
        ]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
        .collect();

        res.extend(self.api.iter());
        res.extend(self.jwt.iter());
        res.extend(self.auth.iter());
        res.extend(self.firewall.iter());
        res.extend(self.html_render.iter());

        res.into_iter()
    }
}

impl LoadArgs for WebConfig {
    fn init_args(&mut self, args: &mut Args) {
        args.add_options([
            ("web-host", None, None),
            ("web-hostname", None, None),
            ("web-base-url", None, None),
            ("web-trusted-hosts", None, None),
            ("web-accept-hosts", None, None),
            ("web-static-dir", None, None),
            ("web-static-path", None, None),
            ("web-static-cache", None, None)
        ])
        .unwrap();

        let list = [
            &mut self.api as &mut dyn LoadArgs,
            &mut self.jwt,
            &mut self.auth,
            &mut self.firewall,
            self.html_render.try_mut().unwrap()
        ];

        for item in list {
            item.init_args(args);
        }
    }

    fn load_args(&mut self, args: &Args) {
        self.extend(
            [
                ("host", args.get("web-host")),
                ("hostname", args.get("web-hostname")),
                ("base_url", args.get("web-base-url")),
                ("trusted_hosts", args.get("web-trusted-hosts")),
                ("accept_hosts", args.get("web-accept-hosts")),
                ("static_dir", args.get("web-static-dir")),
                ("static_path", args.get("web-static-path")),
                ("static_cache", args.get("web-static-cache"))
            ]
            .iter()
            .map(convert::tuple_result_option_str)
        );

        let list = [
            &mut self.api as &mut dyn LoadArgs,
            &mut self.jwt,
            &mut self.auth,
            &mut self.firewall,
            self.html_render.try_mut().unwrap()
        ];

        for item in list {
            item.load_args(args);
        }
    }
}

impl LoadEnv for WebConfig {
    fn load_env(&mut self) {
        self.extend(
            [
                ("host", getenv("WEB_HOST")),
                ("hostname", getenv("WEB_HOSTNAME")),
                ("base_url", getenv("WEB_BASE_URL")),
                ("trusted_hosts", getenv("WEB_TRUSTED_HOSTS")),
                ("accept_hosts", getenv("WEB_ACCEPT_HOSTS")),
                ("static_dir", getenv("WEB_STATIC_DIR")),
                ("static_path", getenv("WEB_STATIC_PATH")),
                ("static_cache", getenv("WEB_STATIC_CACHE"))
            ]
            .iter()
            .map(convert::tuple_option_str)
        );

        let list = [
            &mut self.api as &mut dyn LoadEnv,
            &mut self.jwt,
            &mut self.auth,
            &mut self.firewall,
            self.html_render.try_mut().unwrap()
        ];

        for item in list {
            item.load_env();
        }
    }
}

impl LoadDirs for WebConfig {
    fn load_dirs(&mut self, dirs: &Dirs) {
        if dirs.data.is_empty() == false && self.static_dir.starts_with("/") == false {
            self.static_dir.insert(0, '/');
            self.static_dir.insert_str(0, &dirs.data);
        }

        let list = [self.html_render.try_mut().unwrap() as &mut dyn LoadDirs];

        for item in list {
            item.load_dirs(dirs);
        }
    }
}
