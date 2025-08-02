use {crate::*, app_base::prelude::*};

pub const MODULE_APP: AppModule = module_app;

fn module_app(app: &mut App, event: AppEvent) -> Void {
    match event {
        AppEvent::APP_INIT => {
            app.register_command(AppConfig::DEFAULT_COMMAND, MODULE_APP);
            ok()
        },
        #[cfg(not(feature = "std"))]
        AppEvent::APP_RUN => {
            mem_stats();
            ok()
        },
        #[cfg(feature = "std")]
        AppEvent::APP_RUN => {
            use {
                actix_web::{HttpRequest, HttpResponse, web},
                app_async::actix_with_tokio_start,
                app_web::HttpServer,
                std::sync::Arc
            };

            let config = app.get::<AppConfig>().unwrap();

            actix_with_tokio_start(Some(&config.tokio), async {
                let server = HttpServer::new(&config.actix);
                let mut server_config = HttpServerConfigurator::new(&config);

                server_config.add(|srv, _| {
                    srv.default_service(web::to(|req: HttpRequest| {
                        async move {
                            let body = format!(
                                "URI: {}\n\nAppConfig: {:?}",
                                req.uri(),
                                req.app_data::<Arc<AppConfig>>()
                            );
                            HttpResponse::Ok().body(body)
                        }
                    }));
                });

                server.run(server_config.configure()).await?;

                ok()
            })?
        },
        _ => ok()
    }
}
