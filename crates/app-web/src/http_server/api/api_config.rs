use {
    app_base::prelude::*,
    core::fmt::Display,
    serde::{Deserialize, Serialize}
};

#[derive(Debug, ExtendFromIter, Serialize, Deserialize)]
pub struct ApiConfig {
    pub url: String,
    pub path: String,
    pub proxy_url: String
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:80/api".into(),
            path: "/api".into(),
            proxy_url: "http://127.0.0.1:3000".into()
        }
    }
}

impl Iter<'_, (&'static str, String)> for ApiConfig {
    fn iter(&self) -> impl Iterator<Item = (&'static str, String)> {
        [
            ("web.api.url", &self.url as &dyn Display),
            ("web.api.path", &self.path),
            ("web.api.proxy_url", &self.proxy_url)
        ]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
    }
}

impl LoadArgs for ApiConfig {
    fn init_args(&mut self, args: &mut Args) {
        args.add_options([
            ("web-api-url", None, None),
            ("web-api-path", None, None),
            ("web-api-proxy-url", None, None)
        ])
        .unwrap();
    }

    fn load_args(&mut self, args: &app_base::prelude::Args) {
        self.extend(
            [
                ("url", args.get("web-api-url")),
                ("path", args.get("web-api-path")),
                ("proxy_url", args.get("web-api-proxy-url"))
            ]
            .iter()
            .map(convert::tuple_result_option_str)
        );
    }
}

impl LoadEnv for ApiConfig {
    fn load_env(&mut self) {
        self.extend(
            [
                ("url", getenv("WEB_API_URL")),
                ("path", getenv("WEB_API_PATH")),
                ("proxy_url", getenv("WEB_API_PROXY_URL"))
            ]
            .iter()
            .map(convert::tuple_option_str)
        );
    }
}
