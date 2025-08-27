use {
    app_base::prelude::*,
    serde::{Deserialize, Serialize}
};

#[derive(Debug, ExtendFromIter, Serialize, Deserialize)]
pub struct TokioConfig {
    pub threads: usize,
    pub blocking_threads: usize,
    pub thread_name: String
}

impl Default for TokioConfig {
    fn default() -> Self {
        Self {
            threads: 2,
            blocking_threads: 512,
            thread_name: "tokio-rt worker".into()
        }
    }
}

impl LoadEnv for TokioConfig {
    fn load_env(&mut self) -> Void {
        self.extend(
            [("threads", getenv("TOKIO_THREADS"))]
                .iter()
                .map(convert::tuple_option_str)
        );
        ok()
    }
}

impl LoadArgs for TokioConfig {
    fn load_args(&mut self, args: &Args) -> Void {
        self.extend(
            [("threads", args.get("tokio-threads"))]
                .iter()
                .map(convert::tuple_option_option_str)
        );
        ok()
    }
}
