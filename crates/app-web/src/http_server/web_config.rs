use app_base::prelude::*;

#[derive(Debug, ExtendFromIter)]
pub struct WebConfig {
    pub host: String,
    pub hostname: String,
    pub url: String,
    pub trusted_hosts: Vec<String>,
    pub accept_hosts: Vec<String>,
    pub static_path: String,
    pub static_dir: String
}

impl WebConfig {
    pub fn with_dirs(&mut self, dirs: &Dirs) -> &mut Self {
        if dirs.data.is_empty() == false && self.static_dir.starts_with("/") == false {
            self.static_dir.insert(0, '/');
            self.static_dir.insert_str(0, &dirs.data);
        }
        self
    }
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            host: Default::default(),
            hostname: Default::default(),
            url: Default::default(),
            trusted_hosts: Default::default(),
            accept_hosts: Default::default(),
            static_path: "/public".into(),
            static_dir: "public".into()
        }
    }
}

impl LoadEnv for WebConfig {
    fn load_env(&mut self) -> Void {
        self.extend(
            [
                ("host", getenv("WEB_HOST")),
                ("hostname", getenv("WEB_HOSTNAME")),
                ("url", getenv("WEB_URL")),
                ("trusted_hosts", getenv("WEB_TRUSTED_HOSTS")),
                ("accept_hosts", getenv("WEB_ACCEPT_HOSTS")),
                ("static_dir", getenv("WEB_STATIC_DIR")),
                ("static_path", getenv("WEB_STATIC_PATH"))
            ]
            .iter()
            .map(convert::tuple_option_str)
        );
        ok()
    }
}

impl LoadArgs for WebConfig {
    fn load_args(&mut self, args: &Args) -> Void {
        self.extend(
            [
                ("host", args.get("web-host")),
                ("hostname", args.get("web-hostname")),
                ("url", args.get("web-url")),
                ("trusted_hosts", args.get("web-trusted-hosts")),
                ("accept_hosts", args.get("web-accept-hosts")),
                ("static_dir", args.get("web-static-dir")),
                ("static_path", args.get("web-static-path"))
            ]
            .iter()
            .map(convert::tuple_option_option_str)
        );
        ok()
    }
}
