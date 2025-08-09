use {
    crate::AppConfig,
    actix_files::Files,
    actix_multipart::form::tempfile::TempFileConfig,
    actix_web::web::ServiceConfig,
    alloc::{boxed::Box, sync::Arc, vec::Vec},
    app_async::{
        cache::{ArrayCache, Cacher},
        db::db_pool
    },
    app_base::prelude::*,
    core::pin::Pin,
    futures::executor::block_on,
    sqlx::Postgres
};

type ServiceConfigFn = Pin<Box<dyn Fn(&mut ServiceConfig, &HttpServer) + Send + Sync>>;

pub struct HttpServer {
    pub config: Arc<AppConfig>,
    services: Vec<ServiceConfigFn>
}

impl HttpServer {
    pub fn new(config: &Arc<AppConfig>) -> Self {
        let mut this = Self { config: config.clone(), services: Default::default() };
        this.with_configs()
            .with_db_pool()
            .with_multipart()
            .with_cache()
            .with_static_files();
        this
    }

    pub async fn run(self) -> Void {
        let config = self.config.clone();
        let config_ref = config.clone();
        let configure = Arc::new(self.configure());

        log::info!("Starting HttpServer: {:?}", &config.actix);

        actix_web::HttpServer::new(move || {
            actix_web::App::new()
                /*
                .wrap(GrantsMiddleware::with_extractor(
                    super::middleware::auth_role_extract
                ))
                .wrap(super::middleware::AuthRequired)
                .wrap(from_fn(super::middleware::captcha))
                .wrap(from_fn(super::middleware::firewall))
                .wrap(from_fn(super::middleware::no_cache))
                .wrap(super::middleware::AuthHeader)
                .wrap(super::middleware::errors())
                */
                .wrap(app_web::middleware::cors(config_ref.web.clone()))
                .wrap(actix_web::middleware::NormalizePath::trim())
                .wrap(actix_web::middleware::DefaultHeaders::new())
                .wrap(actix_web::middleware::Logger::default())
                .configure({
                    // !this closure executes for each worker!
                    let configure = configure.clone();
                    move |srv| configure(srv)
                })
        })
        .workers(config.actix.threads as usize)
        .worker_max_blocking_threads(config.actix.blocking_threads_per_worker as usize)
        .bind((config.actix.listen.to_owned(), config.actix.port))?
        .bind_uds(&config.actix.socket)?
        .run()
        .await?;

        ok()
    }

    pub fn add_service(
        &mut self,
        service: impl Fn(&mut ServiceConfig, &HttpServer) + Send + Sync + 'static
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

    fn with_configs(&mut self) -> &mut Self {
        self.add_service(move |srv, cfg| {
            srv.app_data(cfg.config.clone());
            srv.app_data(cfg.config.web.clone());
        })
    }

    fn with_db_pool(&mut self) -> &mut Self {
        self.add_service(move |srv, cfg| {
            srv.app_data(block_on(async {
                db_pool::<Postgres>(Some(&cfg.config.db)).await.unwrap()
            }));
        })
    }

    fn with_multipart(&mut self) -> &mut Self {
        self.add_service(|srv, cfg| {
            srv.app_data(TempFileConfig::default().directory(&cfg.config.dirs.tmp));
        })
    }

    fn with_cache(&mut self) -> &mut Self {
        self.add_service(|srv, _| {
            srv.app_data(Cacher::<ArrayCache>::from_static());
        })
    }

    fn with_static_files(&mut self) -> &mut Self {
        self.add_service(|srv, cfg| {
            srv.service(Files::new(
                &cfg.config.web.static_path, &cfg.config.web.static_dir
            ));
        })
    }
}
