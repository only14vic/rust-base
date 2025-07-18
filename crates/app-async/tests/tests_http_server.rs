/*
use {
    actix_http::StatusCode,
    actix_web::{
        body::MessageBody,
        dev::Service,
        error::{ErrorBadRequest, ErrorUnauthorized},
        test::{call_service, read_body, TestRequest},
        web, HttpRequest, HttpResponse
    },
    app::App,
    app_async::{
        http_server::{HttpServer, HttpServerConfig},
        test_app
    },
    app_base::prelude::*,
    common::TEST
};

mod common;

#[actix_web::test]
async fn test_http_server_success_response() -> Void {
    TEST.run(async {
        let app = App::boot()?;

        let mut server = HttpServer::new(&config.actix);

        server.app_config.add(|srv, _| {
            srv.default_service(web::to(|req: HttpRequest| {
                async move { HttpResponse::Ok().body(req.uri().to_string()) }
            }));
        });

        let app = test_app!(server.app_config);
        let req = TestRequest::with_uri("/foo?bar").to_request();
        let res = app.call(req).await?;

        assert!(res.status().is_success());

        let body = res.into_body().try_into_bytes().unwrap();
        let body = String::from_utf8_lossy(body.as_ref());

        assert_eq!(&body, "/foo?bar");

        ok()
    })
    .await
}
*/
