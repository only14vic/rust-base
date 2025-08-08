use {crate::*, app_base::prelude::*};

pub const MODULE_APP: AppModule = module_app;

fn module_app(app: &mut App, event: AppEvent) -> Void {
    match event {
        AppEvent::APP_INIT => {
            app.register_command(AppConfig::DEFAULT_COMMAND, MODULE_APP);
        },
        AppEvent::APP_RUN => {
            let args = app.get_ref::<Args>().unwrap();
            if args.get("help").unwrap().is_some() {
                show_help(app)?;
            } else {
                #[cfg(feature = "std")]
                server_run(app)?;
                #[cfg(not(feature = "std"))]
                mem_stats();
            }
        },
        _ => ()
    }
    ok()
}

fn show_help(app: &App) -> Void {
    let config = app.config();
    let exe_file = config.dirs.exe_file();

    println!(
        r#"
Usage: {exe_file} [command] [options]

Commands:
    {default}       - run http server (default)
    config    - show config options

Options:
    -h, --help - show usage help
"#,
        default = AppConfig::DEFAULT_COMMAND
    );
    ok()
}

#[cfg(feature = "std")]
fn server_run(app: &mut App) -> Void {
    use {
        actix_web::{HttpRequest, HttpResponse, web},
        app_async::actix_with_tokio_start,
        app_web::{HttpServer, OkHttp},
        std::sync::Arc
    };

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
