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
            dotenv(false);
        }
        .boxed_local()
    })
});

#[macro_export]
macro_rules! test_app {
    ($configurator:expr) => {{
        use {
            //actix_web::middleware::ErrorHandlers,
            actix_web::test::init_service,
            actix_web::App,
            //app_web::http_server::middleware,
            //actix_web_grants::GrantsMiddleware,
        };

        let configure = $configurator.configure();

        init_service(
            App::new()
                //.wrap(GrantsMiddleware::with_extractor(
                //    middleware::auth_role_extract
                //))
                //.wrap(middleware::AuthRequired)
                //.wrap(middleware::AuthHeader)
                //.wrap(middleware::errors())
                .wrap(actix_web::middleware::NormalizePath::trim())
                .wrap(actix_web::middleware::DefaultHeaders::new())
                .configure(move |srv| configure(srv))
        )
        .await
    }};
}
