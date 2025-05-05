use app_base::prelude::*;

#[derive(Debug, SetFromIter)]
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
    fn load_env(&mut self) -> Ok<&mut Self> {
        self.set_from_iter(
            [("threads", getenv("TOKIO_THREADS"))]
                .iter()
                .map(|(k, v)| (*k, v.as_ref().map(String::as_str)))
        )?;
        self.into_ok()
    }
}

impl LoadArgs for TokioConfig {
    fn load_args(&mut self, args: &Args) -> Ok<&mut Self> {
        #[rustfmt::skip]
        self.set_from_iter(
            [
                ("threads", args.get("tokio-threads")),
            ]
            .iter().map(|(k, v)| {(
                *k, v.unwrap_or(&None).as_ref().map(String::as_str)
            )})
        )?;
        self.into_ok()
    }
}
