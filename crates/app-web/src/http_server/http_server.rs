use {
    super::ActixConfig,
    actix_web::{middleware, web::ServiceConfig, App},
    //actix_web::middleware::from_fn,
    //actix_web_grants::GrantsMiddleware,
    app_base::prelude::*,
    std::sync::Arc
};

pub struct HttpServer {
    pub config: Arc<ActixConfig>
}

impl HttpServer {
    pub fn new(config: &Arc<ActixConfig>) -> Self {
        Self { config: config.clone() }
    }

    pub async fn run(
        self,
        configure: impl Fn(&mut ServiceConfig) + Send + Sync + 'static
    ) -> Void {
        log::debug!("Starting HttpServer: {:?}", &self.config);

        let configure = Arc::new(configure);

        actix_web::HttpServer::new(move || {
            App::new()
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
                .wrap(super::middleware::cors())
                */
                .wrap(middleware::NormalizePath::trim())
                .wrap(middleware::DefaultHeaders::new())
                .wrap(middleware::Logger::default())
                .configure({
                    // !this closure executes for each worker!
                    let configure = configure.clone();
                    move |srv| configure(srv)
                })
        })
        .workers(self.config.threads as usize)
        .worker_max_blocking_threads(self.config.blocking_threads_per_worker as usize)
        .bind((self.config.listen.clone(), self.config.port))?
        .bind_uds(&self.config.socket)?
        .run()
        .await?;

        ok()
    }
}
