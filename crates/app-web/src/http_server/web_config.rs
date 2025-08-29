use {
    crate::{
        HtmlRenderConfig,
        api::ApiConfig,
        ext::{AuthConfig, FirewallConfig, JwtConfig}
    },
    app_base::prelude::*,
    core::num::NonZero,
    serde::{Deserialize, Serialize}
};

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
    pub html_render: HtmlRenderConfig
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            host: "localhost:80".into(),
            hostname: "localhost".into(),
            base_url: "http://localhost:80".into(),
            trusted_hosts: vec!["localhost".into()],
            accept_hosts: vec!["localhost".into()],
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

impl LoadDirs for WebConfig {
    fn load_dirs(&mut self, dirs: &Dirs) -> Void {
        if dirs.data.is_empty() == false && self.static_dir.starts_with("/") == false {
            self.static_dir.insert(0, '/');
            self.static_dir.insert_str(0, &dirs.data);
        }
        self.html_render.load_dirs(dirs)?;
        ok()
    }
}

impl LoadEnv for WebConfig {
    fn load_env(&mut self) -> Void {
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
        self.api.load_env()?;
        self.jwt.load_env()?;
        self.auth.load_env()?;
        self.firewall.load_env()?;
        self.html_render.load_env()?;
        ok()
    }
}

impl LoadArgs for WebConfig {
    fn load_args(&mut self, args: &Args) -> Void {
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
            .map(convert::tuple_option_option_str)
        );
        self.api.load_args(args)?;
        self.jwt.load_args(args)?;
        self.auth.load_args(args)?;
        self.firewall.load_args(args)?;
        self.html_render.load_args(args)?;
        ok()
    }
}
