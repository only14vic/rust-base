use app_base::prelude::*;

#[derive(Debug, Default, ExtendFromIter)]
pub struct WebConfig {
    pub host: String,
    pub hostname: String,
    pub url: String,
    pub trusted_hosts: Vec<String>,
    pub accept_hosts: Vec<String>
}

impl LoadEnv for WebConfig {
    fn load_env(&mut self) -> Void {
        self.extend(
            [
                ("host", getenv("WEB_HOST")),
                ("hostname", getenv("WEB_HOSTNAME")),
                ("url", getenv("WEB_URL")),
                ("trusted_hosts", getenv("WEB_TRUSTED_HOSTS")),
                ("accept_hosts", getenv("WEB_ACCEPT_HOSTS"))
            ]
            .iter()
            .map(convert::tuple_option_string_to_str)
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
                ("accept_hosts", args.get("web-accept-hosts"))
            ]
            .iter()
            .map(convert::tuple_option_option_string_to_str)
        );
        ok()
    }
}
