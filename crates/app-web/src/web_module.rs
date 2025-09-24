use {
    crate::{HttpServer, WebConfigExt},
    app_base::prelude::*,
    core::marker::PhantomData,
    futures::executor::block_on
};

pub struct WebModule<C: AppConfigExt> {
    pub enable_defaults: bool,
    pub enable_runtime: bool,
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
            _phantom: PhantomData
        }
    }
}

impl<C> AppModuleExt for WebModule<C>
where
    C: WebConfigExt
{
    type Config = C;

    const COMMAND: &str = "serve";
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
            server.run_with_runtime()
        } else {
            block_on(server.run())
        }
    }
}
