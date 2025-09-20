use {
    crate::{
        ActixConfig, HtmlRender, WebConfig,
        api::api_postgrest,
        ext::{DbWeb, JwtEncoder, RequestExt}
    },
    actix_files::Files,
    actix_multipart::form::tempfile::TempFileConfig,
    actix_web::{
        HttpRequest,
        http::header::ContentType,
        middleware::from_fn,
        web::{self, ServiceConfig}
    },
    actix_web_grants::GrantsMiddleware,
    app_async::{
        TokioConfig, actix_with_tokio_start,
        cache::{ArrayCache, Cacher},
        db::{DbConfig, db_pool}
    },
    app_base::prelude::*,
    core::pin::Pin,
    futures::executor::block_on,
    serde_json::Value,
    sqlx::Postgres,
    std::{boxed::Box, sync::Arc, vec::Vec}
};

type ServiceConfigFn<C> =
    Pin<Box<dyn Fn(&mut ServiceConfig, &HttpServer<C>) + Send + Sync>>;

pub struct HttpServer<C>
where
    C: AppConfigExt + AsRef<Arc<ActixConfig>> + AsRef<Arc<WebConfig>>
{
    pub config: Arc<AppConfig<C>>,
    services: Vec<ServiceConfigFn<C>>
}

impl<C> HttpServer<C>
where
    C: AppConfigExt + AsRef<Arc<ActixConfig>> + AsRef<Arc<WebConfig>>
{
    pub fn new(config: &Arc<AppConfig<C>>) -> Self {
        Self { config: config.clone(), services: Default::default() }
    }

    #[cold]
    pub async fn run(self) -> Void {
        let web_config = self.config.get::<WebConfig>().clone();
        let actix_config = self.config.get::<ActixConfig>().clone();

        log::info!("Starting HttpServer: {:?}", actix_config);

        if actix_config.socket.is_empty() == false {
            Dirs::mkdir(Dirs::dirname(&actix_config.socket))?;
        }

        let configure = Arc::new(self.configure());

        actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .wrap(GrantsMiddleware::with_extractor(
                    super::middleware::auth_role_extract
                ))
                .wrap(super::middleware::AuthRequired)
                //.wrap(from_fn(app_web::middleware::captcha))
                //.wrap(from_fn(app_web::middleware::firewall))
                .wrap(from_fn(super::middleware::cache_control))
                .wrap(super::middleware::AuthHeader)
                //.wrap(super::middleware::errors())
                .wrap(super::middleware::cors(&web_config))
                .wrap(actix_web::middleware::NormalizePath::trim())
                .wrap(actix_web::middleware::DefaultHeaders::new())
                .wrap(actix_web::middleware::Logger::default())
                .configure({
                    // !this closure executes for each worker!
                    let configure = configure.clone();
                    move |srv| configure(srv)
                })
        })
        .workers(actix_config.threads as usize)
        .worker_max_blocking_threads(actix_config.blocking_threads_per_worker as usize)
        .bind((actix_config.listen.to_owned(), actix_config.port))?
        .bind_uds(&actix_config.socket)?
        .run()
        .await?;

        ok()
    }

    pub fn run_with_runtime(self) -> Void
    where
        C: AsRef<Arc<TokioConfig>>
    {
        let tokio_config = self.config.get::<TokioConfig>().clone();
        actix_with_tokio_start(Some(&tokio_config), self.run())?
    }

    pub fn add_service(
        &mut self,
        service: impl Fn(&mut ServiceConfig, &HttpServer<C>) + Send + Sync + 'static
    ) -> &mut Self {
        self.services.push(Box::pin(service));
        self
    }

    pub fn configure(self) -> impl Fn(&mut ServiceConfig) + Send + Sync + 'static {
        move |srv: &mut ServiceConfig| {
            for f in self.services.iter() {
                f(srv, &self);
            }
        }
    }

    pub fn with_defaults(&mut self, app: &mut App<C>) -> &mut Self
    where
        C: AsRef<Arc<DbConfig>>
    {
        self.with_app(app)
            .with_configs()
            .with_jwt()
            .with_db()
            .with_multipart()
            .with_cache()
            .with_static_files()
            .with_html_render()
            .with_api()
            .with_default()
    }

    pub fn with_configs(&mut self) -> &mut Self {
        self.add_service(move |srv, server| {
            srv.app_data(server.config.clone());
            srv.app_data(server.config.base.clone());
            srv.app_data(server.config.dirs.clone());
            srv.app_data(server.config.external.clone());
            srv.app_data(server.config.get::<WebConfig>().clone());
        })
    }

    pub fn with_db(&mut self) -> &mut Self
    where
        C: AsRef<Arc<DbConfig>>
    {
        self.add_service(move |srv, server| {
            let db_config = server.config.get::<DbConfig>();
            let db_pool =
                block_on(async { db_pool::<Postgres>(Some(db_config)).await.unwrap() });
            srv.app_data(Arc::new(DbWeb::new(&db_pool)));
            srv.app_data(db_pool);
        })
    }

    pub fn with_jwt(&mut self) -> &mut Self {
        self.add_service(|srv, server| {
            let web_config = server.config.get::<WebConfig>();
            srv.app_data(JwtEncoder::new(&web_config.jwt));
        })
    }

    pub fn with_multipart(&mut self) -> &mut Self {
        self.add_service(|srv, server| {
            srv.app_data(TempFileConfig::default().directory(&server.config.dirs.tmp));
        })
    }

    pub fn with_cache(&mut self) -> &mut Self {
        self.add_service(|srv, _| {
            srv.app_data(Cacher::<ArrayCache>::from_static());
        })
    }

    pub fn with_static_files(&mut self) -> &mut Self {
        self.add_service(|srv, server| {
            let web_config = server.config.get::<WebConfig>();
            srv.service(Files::new(&web_config.static_path, &web_config.static_dir));
        })
    }

    pub fn with_html_render(&mut self) -> &mut Self {
        let web_config = self.config.get::<WebConfig>();
        let html_render: Arc<_> = HtmlRender::new(&web_config.html_render).into();

        self.add_service(move |srv, _server| {
            srv.app_data(html_render.clone());
        })
    }

    pub fn with_data<T: Send + Sync + 'static>(&mut self, data: T) -> &mut Self {
        let data: Arc<_> = data.into();
        self.add_service(move |srv, _server| {
            srv.app_data(data.clone());
        })
    }

    pub fn with_app(&mut self, app: &mut App<C>) -> &mut Self {
        let app = unsafe { &*(app as *const App<C>) };
        let di = unsafe { &*(&**app as *const Di) };
        self.add_service(|srv, _server| {
            srv.app_data::<&'static App<C>>(app);
            srv.app_data::<&'static Di>(di);
        })
    }

    pub fn with_api(&mut self) -> &mut Self {
        self.add_service(|srv, server| {
            let web_config = server.config.get::<WebConfig>();
            srv.service({
                web::scope(&web_config.api.path)
                    .wrap(from_fn(
                        super::middleware::content_type(ContentType::json())
                    ))
                    .default_service(web::to(api_postgrest))
            });
        })
    }

    pub fn with_default(&mut self) -> &mut Self {
        self.add_service(|srv, _server| {
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
        })
    }
}
