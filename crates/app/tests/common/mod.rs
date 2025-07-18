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
    ($config:expr) => {{
        use {
            actix_web::middleware::ErrorHandlers,
            actix_web::test::init_service,
            actix_web::App,
            app_web::http_server::HttpServer,
            app_web::http_server::middleware,
            //actix_web_grants::GrantsMiddleware,
        };

        let configure = $config.into_configure();

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

    () => {{
        let server = app_web::http_server::HttpServer::default();
        test_app!(server.app_config)
    }};
}
