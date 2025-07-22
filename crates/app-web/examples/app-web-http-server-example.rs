use {
    actix_web::{web, HttpRequest, HttpResponse},
    app::{AppConfig, HttpServerConfigurator},
    app_async::actix_with_tokio_start,
    app_base::prelude::*,
    app_web::HttpServer,
    std::sync::Arc
};

fn main() -> Void {
    let app = app::App::boot()?;
    let config = app.get::<AppConfig>().unwrap();

    actix_with_tokio_start(Some(&config.tokio), async {
        let server = HttpServer::new(&config.actix);
        let mut server_config = HttpServerConfigurator::new(&config);

        server_config.add(|srv, _| {
            srv.default_service(web::to(|req: HttpRequest| {
                let body = format!(
                    "URI: {}\n\nAppConfig: {:?}",
                    req.uri(),
                    req.app_data::<Arc<AppConfig>>()
                );
                async move { HttpResponse::Ok().body(body) }
            }));
        });

        server.run(server_config.configure()).await
    })?
}
