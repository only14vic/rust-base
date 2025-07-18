use {
    crate::AppConfig,
    actix_web::web::ServiceConfig,
    alloc::{boxed::Box, sync::Arc, vec::Vec},
    core::pin::Pin
};

type ServiceConfigFn =
    Pin<Box<dyn Fn(&mut ServiceConfig, &HttpServerConfigurator) + Send + Sync>>;

pub struct HttpServerConfigurator {
    pub config: Arc<AppConfig>,
    services: Vec<ServiceConfigFn>
}

impl HttpServerConfigurator {
    pub fn new(config: Arc<AppConfig>) -> Self {
        let mut this = Self { config, services: Default::default() };
        this.with_app_config();
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
        self.add(move |srv, configurator| {
            srv.app_data(configurator.config.clone());
        })
    }
}
