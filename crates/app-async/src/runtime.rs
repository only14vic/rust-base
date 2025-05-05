use {
    crate::TokioConfig,
    std::{future::Future, sync::LazyLock}
};

pub fn tokio_start(
    config: Option<&dyn AsRef<TokioConfig>>
) -> std::io::Result<tokio::runtime::Runtime> {
    static DEFAULT_CONFIG: LazyLock<TokioConfig> = LazyLock::new(Default::default);

    let config = config
        .as_ref()
        .map(|c| c.as_ref())
        .unwrap_or(&DEFAULT_CONFIG);

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(config.threads)
        .max_blocking_threads(config.blocking_threads)
        .thread_name(&config.thread_name)
        .enable_all()
        .build()
}

pub fn actix_with_tokio_start<T>(
    config: Option<&dyn AsRef<TokioConfig>>,
    fut: impl Future<Output = T>
) -> std::io::Result<T> {
    let rt = tokio_start(config)?;
    let res = actix::System::with_tokio_rt(|| rt).block_on(fut);

    Ok(res)
}
