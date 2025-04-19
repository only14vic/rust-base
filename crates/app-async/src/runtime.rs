use {crate::TokioConfig, std::future::Future};

pub fn tokio_start(
    config: Option<&TokioConfig>
) -> std::io::Result<tokio::runtime::Runtime> {
    let config_default = Default::default();
    let config = config.unwrap_or(&config_default);

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(config.worker_threads)
        .max_blocking_threads(config.max_blocking_threads)
        .thread_name(&config.thread_name)
        .enable_all()
        .build()
}

pub fn actix_on_tokio_start<T>(
    config: Option<&TokioConfig>,
    fut: impl Future<Output = T>
) -> std::io::Result<T> {
    let rt = tokio_start(config)?;
    let res = actix::System::with_tokio_rt(|| rt).block_on(fut);

    Ok(res)
}
