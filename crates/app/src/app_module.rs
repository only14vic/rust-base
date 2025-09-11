use {crate::*, app_base::prelude::*};

pub type App = app_base::prelude::App<Config>;
pub type AppConfig = app_base::prelude::AppConfig<Config>;

pub const MODULE_APP: AppModule<Config> = module_app;
pub const MODULE_APP_CONFIG: AppModule<Config> = module_app_config;

fn module_app(app: &mut App, event: AppEvent) -> Void {
    match event {
        AppEvent::APP_INIT => {
            app.register_command(Config::DEFAULT_COMMAND, MODULE_APP);
        },
        AppEvent::APP_RUN => {
            let args = app.get_ref::<Args>().unwrap();
            if args.get("help").unwrap().is_some() {
                show_help(app)?;
            } else {
                let config = app.config();
                Dirs::mkdir(&config.dirs.var)?;
                Dirs::mkdir(&config.dirs.run)?;
                Dirs::mkdir(&config.dirs.log)?;
                Dirs::mkdir(&config.dirs.tmp)?;
                Dirs::mkdir(&config.dirs.cache)?;
                Dirs::mkdir(&config.dirs.state)?;
                Dirs::mkdir(&config.dirs.user_config)?;

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
        default = Config::DEFAULT_COMMAND
    );
    ok()
}

#[cfg(feature = "std")]
fn server_run(app: &mut App) -> Void {
    use {
        actix_web::{HttpRequest, http::header::ContentType, middleware::from_fn, web},
        app_async::actix_with_tokio_start,
        app_web::{api::api_postgrest, ext::RequestExt, middleware::content_type},
        serde_json::Value
    };

    let config = app.config();

    actix_with_tokio_start(Some(&config.tokio), async {
        let mut server = HttpServer::new(config);

        server.add_service(|srv, cfg| {
            srv.service({
                web::scope(&cfg.config.web.api.path)
                    .wrap(from_fn(content_type(ContentType::json())))
                    .default_service(web::to(api_postgrest))
            });

            srv.default_service(web::to(
                |req: HttpRequest, data: Option<web::Json<Value>>| {
                    async move {
                        let context = req.html_render_context().await?;
                        context.add("data", &data.map(web::Json::into_inner).unwrap_or_default());
                        req.html_render().await
                    }
                }
            ));
        });

        server.run().await
    })?
}
