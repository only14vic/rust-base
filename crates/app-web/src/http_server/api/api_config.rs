use {
    app_base::prelude::*,
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

impl LoadEnv for ApiConfig {
    fn load_env(&mut self) -> app_base::prelude::Void {
        self.extend(
            [
                ("url", getenv("WEB_API_URL")),
                ("path", getenv("WEB_API_PATH")),
                ("proxy_url", getenv("WEB_API_PROXY_URL"))
            ]
            .iter()
            .map(convert::tuple_option_str)
        );
        ok()
    }
}

impl LoadArgs for ApiConfig {
    fn load_args(&mut self, args: &app_base::prelude::Args) -> app_base::prelude::Void {
        self.extend(
            [
                ("url", args.get("web-api-url")),
                ("path", args.get("web-api-path")),
                ("proxy_url", args.get("web-api-proxy-url"))
            ]
            .iter()
            .map(convert::tuple_result_option_str)
        );
        ok()
    }
}
