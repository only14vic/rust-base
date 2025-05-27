use {app_testing::Test, futures_lite::FutureExt, std::sync::LazyLock};

pub static TEST: LazyLock<Test> = LazyLock::new(|| {
    Test::new()
        .init(|| async {}.boxed_local())
        .before(|| async {}.boxed_local())
        .after(|| async {}.boxed_local())
});
