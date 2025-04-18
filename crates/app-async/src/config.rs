use app_base::SetFromIter;

#[derive(Debug, SetFromIter)]
pub struct TokioConfig {
    pub worker_threads: usize,
    pub max_blocking_threads: usize,
    pub thread_name: String
}

impl Default for TokioConfig {
    fn default() -> Self {
        Self {
            worker_threads: 2,
            max_blocking_threads: 512,
            thread_name: "tokio-rt worker".into()
        }
    }
}
