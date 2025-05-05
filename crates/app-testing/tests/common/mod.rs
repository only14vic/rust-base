use {
    app_base::prelude::{dotenv, log_init},
    app_testing::Test,
    futures_lite::FutureExt,
    std::sync::LazyLock
};

pub static TEST: LazyLock<Test> = LazyLock::new(|| {
    Test::new()
        .init(|| {
            async {
                dotenv(false);
                log_init();
                log::debug!("init");
            }
            .boxed_local()
        })
        .before(|| {
            async {
                log::debug!("before");
            }
            .boxed_local()
        })
        .after(|| {
            async {
                log::debug!("after");
            }
            .boxed_local()
        })
});
