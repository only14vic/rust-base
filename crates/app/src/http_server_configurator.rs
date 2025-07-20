use {
    crate::AppConfig,
    actix_files::Files,
    actix_multipart::form::tempfile::TempFileConfig,
    actix_web::web::ServiceConfig,
    alloc::{boxed::Box, sync::Arc, vec::Vec},
    app_async::cache::{ArrayCache, Cacher},
    core::pin::Pin
};

type ServiceConfigFn =
    Pin<Box<dyn Fn(&mut ServiceConfig, &HttpServerConfigurator) + Send + Sync>>;

pub struct HttpServerConfigurator {
    pub config: Arc<AppConfig>,
    services: Vec<ServiceConfigFn>
}

impl HttpServerConfigurator {
    pub fn new(config: &Arc<AppConfig>) -> Self {
        let mut this = Self { config: config.clone(), services: Default::default() };
        this.with_app_config()
            .with_multipart()
            .with_cache()
            .with_static_files();
        this
    }

    pub fn add(
        &mut self,
        service: impl Fn(&mut ServiceConfig, &HttpServerConfigurator) + Send + Sync + 'static
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

    fn with_app_config(&mut self) -> &mut Self {
        self.add(move |srv, cfg| {
            srv.app_data(cfg.config.clone());
        })
    }

    fn with_multipart(&mut self) -> &mut Self {
        self.add(|srv, cfg| {
            srv.app_data(TempFileConfig::default().directory(&cfg.config.dirs.tmp));
        })
    }

    fn with_cache(&mut self) -> &mut Self {
        self.add(|srv, _| {
            srv.app_data(Cacher::<ArrayCache>::from_static());
        })
    }

    fn with_static_files(&mut self) -> &mut Self {
        self.add(|srv, cfg| {
            srv.service(Files::new(
                &cfg.config.actix.static_path,
                cfg.config.dirs.data.to_owned() + &cfg.config.actix.static_path
            ));
        })
    }
}
