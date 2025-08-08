use app_base::prelude::*;

#[derive(Debug, ExtendFromIter)]
pub struct ActixConfig {
    pub socket: String,
    pub listen: String,
    pub port: u16,
    pub threads: u8,
    pub blocking_threads_per_worker: u16,
    pub static_path: String,
    pub static_dir: String
}

impl ActixConfig {
    pub fn with_dirs(&mut self, dirs: &Dirs) -> &mut Self {
        if dirs.data.is_empty() == false && self.static_dir.starts_with("/") == false {
            self.static_dir.insert(0, '/');
            self.static_dir.insert_str(0, &dirs.data);
        }
        if dirs.run.is_empty() == false && self.socket.starts_with("/") == false {
            self.socket.insert(0, '/');
            self.socket.insert_str(0, &dirs.run);
        }
        self
    }
}

impl Default for ActixConfig {
    fn default() -> Self {
        Self {
            socket: "rust-http-server.sock".into(),
            listen: "0.0.0.0".into(),
            port: 80,
            threads: 4,
            blocking_threads_per_worker: 4,
            static_path: "/public".into(),
            static_dir: "public".into()
        }
    }
}

impl LoadEnv for ActixConfig {
    fn load_env(&mut self) -> Void {
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
        ok()
    }
}

impl LoadArgs for ActixConfig {
    fn load_args(&mut self, args: &Args) -> Void {
        self.extend(
            [
                ("socket", args.get("actix-socket")),
                ("listen", args.get("actix-listen")),
                ("port", args.get("actix-port")),
                ("threads", args.get("actix-threads"))
            ]
            .iter()
            .map(convert::tuple_option_option_str)
        );
        ok()
    }
}
