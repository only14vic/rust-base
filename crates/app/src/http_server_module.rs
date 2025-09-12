use {
    crate::{App, *},
    actix_web::{HttpRequest, http::header::ContentType, middleware::from_fn, web},
    app_async::actix_with_tokio_start,
    app_base::prelude::*,
    app_web::{api::api_postgrest, ext::RequestExt, middleware::content_type},
    core::marker::PhantomData,
    serde_json::Value
};

#[derive(Default)]
pub struct HttpServerModule<C: AppConfigExt>(PhantomData<C>);

impl AppModuleExt for HttpServerModule<Config> {
    const COMMAND: &str = Config::DEFAULT_COMMAND;
    const DESCRIPTION: &str = "run http server (default)";

    type Config = Config;

    fn run(&mut self, app: &mut AppBase<Self::Config>) -> Void {
        let config = app.config();

        Dirs::mkdir(&config.dirs.var)?;
        Dirs::mkdir(&config.dirs.run)?;
        Dirs::mkdir(&config.dirs.log)?;
        Dirs::mkdir(&config.dirs.tmp)?;
        Dirs::mkdir(&config.dirs.cache)?;
        Dirs::mkdir(&config.dirs.state)?;
        Dirs::mkdir(&config.dirs.user_config)?;

        self.server_run(app)?;

        ok()
    }

    fn help(&self, app: &mut AppBase<Self::Config>) -> Void {
        let config = app.config();

        println!(
            r#"
Usage: {bin} [{cmd}] [options]

Commands:
    {cmd:<len$} - {desc}
    {cfg:<len$} - {cfg_desc}
    {mtr:<len$} - {mtr_desc}

Options:
    -h, --help - show usage help
"#,
            len = 10,
            bin = config.dirs.exe_file(),
            cmd = Self::COMMAND,
            desc = Self::DESCRIPTION,
            cfg = AppConfigModule::<Self::Config>::COMMAND,
            cfg_desc = AppConfigModule::<Self::Config>::DESCRIPTION,
            mtr = MigratorModule::<Self::Config>::COMMAND,
            mtr_desc = MigratorModule::<Self::Config>::DESCRIPTION
        );

        ok()
    }
}

impl HttpServerModule<Config> {
    fn server_run(&self, app: &mut App) -> Void {
        // Make app as static reference
        let app = unsafe { &mut *(app as *mut App) };
        let config = app.config();

        actix_with_tokio_start(Some(&config.tokio), async {
            let mut server = HttpServer::new(config);

            server.add_service(|srv, server| {
                srv.app_data::<&'static App>(app);

                srv.service({
                    web::scope(&server.config.web.api.path)
                        .wrap(from_fn(content_type(ContentType::json())))
                        .default_service(web::to(api_postgrest))
                });

                srv.default_service(web::to(
                    |req: HttpRequest, data: Option<web::Json<Value>>| {
                        async move {
                            let context = req.html_render_context().await?;
                            context.add(
                                "data",
                                &data.map(web::Json::into_inner).unwrap_or_default()
                            );
                            req.html_render().await
                        }
                    }
                ));
            });

            server.run().await
        })?
    }
}
