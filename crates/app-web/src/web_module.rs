use {
    crate::{HttpServer, WebConfig, WebConfigExt},
    app_base::prelude::*,
    core::marker::PhantomData,
    futures::{executor::block_on, future::LocalBoxFuture},
    std::sync::Arc
};

pub struct WebModule<C: WebConfigExt> {
    pub enable_defaults: bool,
    pub enable_runtime: bool,
    init_runtime:
        Option<Arc<dyn Fn() -> LocalBoxFuture<'static, Void> + Send + Sync + 'static>>,
    _phantom: PhantomData<C>
}

impl<C> Default for WebModule<C>
where
    C: WebConfigExt
{
    fn default() -> Self {
        Self {
            enable_defaults: true,
            enable_runtime: true,
            init_runtime: None,
            _phantom: PhantomData
        }
    }
}

impl<C> AppModuleExt for WebModule<C>
where
    C: WebConfigExt
{
    type Config = C;

    const COMMAND: &str = WebConfig::COMMAND;
    const DESCRIPTION: &str = "starts http server";

    fn boot(&mut self, app: &mut App<Self::Config>) -> Void {
        let mut server = HttpServer::new(app.config());
        server.with_app(app);
        app.add(server);
        ok()
    }

    fn run(&mut self, app: &mut App<Self::Config>) -> Void {
        let mut server = app.take::<HttpServer<Self::Config>>().unwrap();

        if self.enable_defaults {
            server.with_defaults();
        }

        if self.enable_runtime {
            let init = self.init_runtime.as_ref().map(|f| f());
            server.run_with_runtime(init)
        } else {
            block_on(server.run())
        }
    }
}

impl<C> WebModule<C>
where
    C: WebConfigExt
{
    pub fn with_init_runtime(
        &mut self,
        init: impl Fn() -> LocalBoxFuture<'static, Void> + Send + Sync + 'static
    ) -> &mut Self {
        self.init_runtime = Some(Arc::new(init));
        self
    }
}
