use {
    actix_web::{
        HttpRequest, HttpResponse,
        body::MessageBody,
        dev::Service,
        test::TestRequest,
        web::{self, ServiceConfig}
    },
    app_base::prelude::*,
    common::TEST
};

mod common;

#[actix_web::test]
async fn test_http_server_success_response() -> Void {
    TEST.run(async {
        let configure = |srv: &mut ServiceConfig| {
            srv.default_service(web::to(|req: HttpRequest| {
                async move { HttpResponse::Ok().body(req.uri().to_string()) }
            }));
        };

        let app = test_app!(configure);
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
