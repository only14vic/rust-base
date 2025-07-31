#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;
extern crate core;

#[cfg(not(feature = "std"))]
use core::ffi::{c_char, c_int};

#[allow(unused_imports)]
use {app::*, app_base::prelude::*};

static MODULE_APP: fn(&mut App) -> Void = module_init;

fn module_init(_app: &mut App) -> Void {
    log::trace!("App module loaded");
    ok()
}

#[cfg(feature = "std")]
fn main() -> Void {
    use {
        actix_web::{HttpRequest, HttpResponse, web},
        app_async::actix_with_tokio_start,
        app_web::HttpServer,
        std::sync::Arc
    };

    let mut app = App::boot().inspect_err(|e| log::error!("{e}"))?;
    app.register_module(MODULE_APP);
    app.load_modules()?;

    let config = app.get::<AppConfig>().unwrap();

    actix_with_tokio_start(Some(&config.tokio), async {
        let server = HttpServer::new(&config.actix);
        let mut server_config = HttpServerConfigurator::new(&config);

        server_config.add(|srv, _| {
            srv.default_service(web::to(|req: HttpRequest| {
                async move {
                    let body = format!(
                        "URI: {}\n\nAppConfig: {:?}",
                        req.uri(),
                        req.app_data::<Arc<AppConfig>>()
                    );
                    HttpResponse::Ok().body(body)
                }
            }));
        });

        server.run(server_config.configure()).await?;

        app.run()
    })
    .inspect_err(|e| log::error!("{e}"))?
}

#[cfg(not(feature = "std"))]
#[unsafe(no_mangle)]
fn main(argc: c_int, argv: *const *const c_char) -> c_int {
    let mut app = App::boot(argc, argv)
        .inspect_err(|e| panic!("{e}"))
        .unwrap();
    let _ = app.run().inspect_err(|e| panic!("{e}"));

    libc::EXIT_SUCCESS
}
