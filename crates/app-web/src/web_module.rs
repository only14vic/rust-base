use {
    crate::{ActixConfig, HttpServer, WebConfig},
    app_async::{TokioConfig, db::DbConfig},
    app_base::prelude::*,
    core::marker::PhantomData,
    std::sync::Arc
};

#[derive(Default)]
pub struct WebModule<C: AppConfigExt>(PhantomData<C>);

impl<C> AppModuleExt for WebModule<C>
where
    C: AppConfigExt
        + AsRef<Arc<TokioConfig>>
        + AsRef<Arc<ActixConfig>>
        + AsRef<Arc<WebConfig>>
        + AsRef<Arc<DbConfig>>
{
    type Config = C;

    const COMMAND: &str = "serve";
    const DESCRIPTION: &str = "starts http server";

    fn boot(&mut self, app: &mut App<Self::Config>) -> Void {
        let server = HttpServer::new(app.config());
        app.add(server);
        ok()
    }

    fn run(&mut self, app: &mut App<Self::Config>) -> Void {
        let mut server = app.take::<HttpServer<Self::Config>>().unwrap();
        server.with_defaults(app);
        server.run_with_runtime()
    }
}
