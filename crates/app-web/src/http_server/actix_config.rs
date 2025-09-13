use {
    app_base::prelude::*,
    core::fmt::Display,
    serde::{Deserialize, Serialize}
};

#[derive(Debug, ExtendFromIter, Serialize, Deserialize)]
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

impl Iter<'_, (&'static str, String)> for ActixConfig {
    fn iter(&self) -> impl Iterator<Item = (&'static str, String)> {
        [
            ("actix.port", &self.port as &dyn Display),
            ("actix.socket", &self.socket),
            ("actix.listen", &self.listen),
            ("actix.threads", &self.threads),
            (
                "actix.blocking_threads_per_worker", &self.blocking_threads_per_worker
            )
        ]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
    }
}

impl LoadArgs for ActixConfig {
    fn init_args(&mut self, args: &mut Args) {
        args.add_options([
            ("actix-socket", None, None),
            ("actix-listen", None, None),
            ("actix-port", None, None),
            ("actix-threads", None, None)
        ])
        .unwrap();
    }

    fn load_args(&mut self, args: &Args) {
        self.extend(
            [
                ("socket", args.get("actix-socket")),
                ("listen", args.get("actix-listen")),
                ("port", args.get("actix-port")),
                ("threads", args.get("actix-threads"))
            ]
            .iter()
            .map(convert::tuple_result_option_str)
        );
    }
}

impl LoadDirs for ActixConfig {
    fn load_dirs(&mut self, dirs: &Dirs) {
        if dirs.run.is_empty() == false && self.socket.starts_with("/") == false {
            self.socket.insert(0, '/');
            self.socket.insert_str(0, &dirs.run);
        }
    }
}

impl LoadEnv for ActixConfig {
    fn load_env(&mut self) {
        self.extend(
            [
                ("socket", getenv("ACTIX_SOCKET")),
                ("listen", getenv("ACTIX_LISTEN")),
                ("port", getenv("ACTIX_PORT")),
                ("threads", getenv("ACTIX_THREADS"))
            ]
            .iter()
            .map(convert::tuple_option_str)
        );
    }
}
