use {
    super::ActixConfig,
    crate::WebConfig,
    actix_web::{App, middleware, web::ServiceConfig},
    app_base::prelude::*,
    std::sync::Arc
};

pub struct HttpServer {
    pub actix_config: Arc<ActixConfig>,
    pub web_config: Arc<WebConfig>
}

impl HttpServer {
    pub fn new(actix_config: &Arc<ActixConfig>, web_config: &Arc<WebConfig>) -> Self {
        Self {
            actix_config: actix_config.clone(),
            web_config: web_config.clone()
        }
    }

    pub async fn run(self, configure: impl Fn(&mut ServiceConfig) + Send + Sync + 'static) -> Void {
        log::info!("Starting HttpServer: {:?}", &self.actix_config);

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
                */
                .wrap(super::middleware::cors(self.web_config.clone()))
                .wrap(middleware::NormalizePath::trim())
                .wrap(middleware::DefaultHeaders::new())
                .wrap(middleware::Logger::default())
                .configure({
                    // !this closure executes for each worker!
                    let configure = configure.clone();
                    move |srv| configure(srv)
                })
        })
        .workers(self.actix_config.threads as usize)
        .worker_max_blocking_threads(self.actix_config.blocking_threads_per_worker as usize)
        .bind((self.actix_config.listen.clone(), self.actix_config.port))?
        .bind_uds(&self.actix_config.socket)?
        .run()
        .await?;

        ok()
    }
}
