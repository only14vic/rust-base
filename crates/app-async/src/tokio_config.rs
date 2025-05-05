use {
    app_base::prelude::*,
    core::fmt::Display,
    serde::{Deserialize, Serialize}
};

#[derive(Debug, ExtendFromIter, Serialize, Deserialize)]
pub struct TokioConfig {
    pub threads: usize,
    pub blocking_threads: usize,
    pub thread_name: String
}

impl AppConfigExt for TokioConfig {}

impl Default for TokioConfig {
    fn default() -> Self {
        Self {
            threads: 2,
            blocking_threads: 512,
            thread_name: "tokio-rt worker".into()
        }
    }
}

impl Iter<'_, (&'static str, String)> for TokioConfig {
    fn iter(&self) -> impl Iterator<Item = (&'static str, String)> {
        [
            ("tokio.threads", &self.threads as &dyn Display),
            ("tokio.blocking_threads", &self.blocking_threads),
            ("tokio.thread_name", &self.thread_name)
        ]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
    }
}

impl LoadArgs for TokioConfig {
    fn init_args(&mut self, args: &mut Args) {
        args.add_options([
            ("tokio-threads", None, None),
            ("tokio-blocking-threads", None, None),
            ("tokio-thread-name", None, None)
        ])
        .unwrap();
    }

    fn load_args(&mut self, args: &Args) {
        self.extend(
            [
                ("threads", args.get("tokio-threads")),
                ("blocking_threads", args.get("tokio-blocking-threads")),
                ("thread_name", args.get("tokio-thread-name"))
            ]
            .iter()
            .map(convert::tuple_result_option_str)
        );
    }
}

impl LoadEnv for TokioConfig {
    fn load_env(&mut self) {
        self.extend(
            [("threads", getenv("TOKIO_THREADS"))]
                .iter()
                .map(convert::tuple_option_str)
        );
    }
}

impl LoadDirs for TokioConfig {
    fn load_dirs<'a>(&'a mut self, _dirs: &'a Dirs) {}
}
