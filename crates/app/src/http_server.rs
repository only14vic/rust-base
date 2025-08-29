use {
    crate::AppConfig,
    actix_files::Files,
    actix_multipart::form::tempfile::TempFileConfig,
    actix_web::{middleware::from_fn, web::ServiceConfig},
    actix_web_grants::GrantsMiddleware,
    alloc::{boxed::Box, sync::Arc, vec::Vec},
    app_async::{
        cache::{ArrayCache, Cacher},
        db::db_pool
    },
    app_base::prelude::*,
    app_web::{
        HtmlRender,
        ext::{DbWeb, JwtEncoder}
    },
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
            .with_jwt()
            .with_db()
            .with_multipart()
            .with_cache()
            .with_static_files()
            .with_html_render();
        this
    }

    pub async fn run(self) -> Void {
        let config = self.config.clone();
        let config_ref = config.clone();
        let configure = Arc::new(self.configure());

        log::info!("Starting HttpServer: {:?}", &config.actix);

        if config.actix.socket.is_empty() == false {
            Dirs::mkdir(Dirs::dirname(&config.actix.socket))?;
        }

        actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .wrap(GrantsMiddleware::with_extractor(
                    app_web::middleware::auth_role_extract
                ))
                .wrap(app_web::middleware::AuthRequired)
                //.wrap(from_fn(app_web::middleware::captcha))
                //.wrap(from_fn(app_web::middleware::firewall))
                .wrap(from_fn(app_web::middleware::cache_control))
                .wrap(app_web::middleware::AuthHeader)
                //.wrap(super::middleware::errors())
                .wrap(app_web::middleware::cors(&config_ref.web))
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
            srv.app_data(cfg.config.base.clone());
            srv.app_data(cfg.config.web.clone());
        })
    }

    fn with_db(&mut self) -> &mut Self {
        self.add_service(move |srv, cfg| {
            let db_pool =
                block_on(async { db_pool::<Postgres>(Some(&cfg.config.db)).await.unwrap() });
            srv.app_data(Arc::new(DbWeb::new(&db_pool)));
            srv.app_data(db_pool);
        })
    }

    fn with_jwt(&mut self) -> &mut Self {
        self.add_service(|srv, cfg| {
            srv.app_data(JwtEncoder::new(&cfg.config.web.jwt));
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

    fn with_html_render(&mut self) -> &mut Self {
        self.add_service(|srv, cfg| {
            srv.app_data(HtmlRender::new(&cfg.config.web.html_render));
        })
    }
}
