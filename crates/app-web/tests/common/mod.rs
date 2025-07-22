use {
    app_base::prelude::*,
    app_testing::Test,
    futures::FutureExt,
    std::{env::set_current_dir, sync::LazyLock}
};

pub static TEST: LazyLock<Test> = LazyLock::new(|| {
    Test::new().init(|| {
        async {
            set_current_dir(env!("PWD")).unwrap();
            Ini::dotenv(&".env.test", true).unwrap();
        }
        .boxed_local()
    })
});

#[macro_export]
macro_rules! test_app {
    ($configure:expr) => {{
        use actix_web::{test::init_service, App};

        let configure = $configure;

        init_service(
            App::new()
                .wrap(actix_web::middleware::NormalizePath::trim())
                .wrap(actix_web::middleware::DefaultHeaders::new())
                .configure(move |srv| configure(srv))
        )
        .await
    }};
}
