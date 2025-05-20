use app_base::prelude::*;

#[derive(Debug, Extend)]
pub struct ActixConfig {
    pub socket: String,
    pub listen: String,
    pub port: u16,
    pub threads: u8,
    pub blocking_threads_per_worker: u16
}

impl Default for ActixConfig {
    fn default() -> Self {
        Self {
            socket: "rust-http-server.sock".into(),
            listen: "0.0.0.0".into(),
            port: 80,
            threads: 4,
            blocking_threads_per_worker: 4
        }
    }
}

impl LoadEnv for ActixConfig {
    fn load_env(&mut self) -> Ok<()> {
        self.extend(
            [
                ("socket", getenv("ACTIX_SOCKET")),
                ("listen", getenv("ACTIX_LISTEN")),
                ("port", getenv("ACTIX_PORT")),
                ("threads", getenv("ACTIX_THREADS"))
            ]
            .iter()
            .map(|(k, v)| (*k, v.as_ref().map(String::as_str)))
        );
        ok()
    }
}

impl LoadArgs for ActixConfig {
    fn load_args(&mut self, args: &Args) -> Ok<()> {
        #[rustfmt::skip]
        self.extend(
            [
                ("socket", args.get("actix-socket")),
                ("listen", args.get("actix-listen")),
                ("port", args.get("actix-port")),
                ("threads", args.get("actix-threads"))
            ]
            .iter()
            .map(|(k, v)| {(
                *k, v.unwrap_or(&None).as_ref().map(String::as_str)
            )})
        );
        ok()
    }
}
