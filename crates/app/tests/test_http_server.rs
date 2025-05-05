use {
    actix_web::{
        HttpRequest, HttpResponse, body::MessageBody, dev::Service, test::TestRequest,
        web
    },
    app::{App, Config},
    app_base::prelude::*,
    app_web::HttpServer,
    common::TEST
};

mod common;

#[actix_web::test]
async fn test_http_server_success() -> Void {
    TEST.run(async {
        let mut app = App::new([]);
        app.boot()?;

        let config = app.config();
        let mut server_config = HttpServer::<Config>::new(&config);

        server_config.add_service(|srv, _cfg| {
            srv.default_service(web::to(|req: HttpRequest| {
                async move { HttpResponse::Ok().body(req.uri().to_string()) }
            }));
        });

        let app = test_app!(server_config);
        let req = TestRequest::with_uri("/foo?bar=1").to_request();
        let res = app.call(req).await?;

        assert!(res.status().is_success());

        let body = res.into_body().try_into_bytes().unwrap();
        let body = String::from_utf8_lossy(body.as_ref());

        assert_eq!(&body, "/foo?bar=1");

        ok()
    })
    .await
}
