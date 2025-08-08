use {
    actix_web::{HttpRequest, HttpResponse, web},
    app::{AppConfig, HttpServerConfigurator},
    app_async::actix_with_tokio_start,
    app_base::prelude::*,
    app_web::{HttpServer, OkHttp},
    std::sync::Arc
};

fn main() -> Void {
    let mut app = app::App::new([]);
    app.boot()?;
    let config = app.get::<AppConfig>().unwrap();

    actix_with_tokio_start(Some(&config.tokio), async {
        let server = HttpServer::new(&config.actix, &config.web);
        let mut server_config = HttpServerConfigurator::new(&config);

        server_config.add(|srv, _| {
            srv.default_service(web::to(|req: HttpRequest| {
                async move {
                    let body = format!(
                        "URI: {}\n\nAppConfig: {:?}",
                        req.uri(),
                        req.app_data::<Arc<AppConfig>>()
                    );

                    HttpResponse::Ok().body(body).into_ok() as OkHttp
                }
            }));
        });

        server.run(server_config.configure()).await
    })?
}
