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

#[cfg(feature = "std")]
fn main() -> Void {
    use {
        actix_web::{web, HttpRequest, HttpResponse},
        app_async::{actix_with_tokio_start, http_server::HttpServer},
        std::sync::Arc
    };

    let mut app = App::boot()?;
    app.run()?;
    let config = app.config();

    actix_with_tokio_start(Some(&config.tokio), async {
        let server = HttpServer::new(&config.actix);
        let mut server_config =
            HttpServerConfigurator::new(app.get::<AppConfig>().unwrap());

        server_config.add(|srv, _| {
            srv.default_service(web::to(|req: HttpRequest| {
                let body = format!(
                    "URI: {}\n\nAppConfig: {:?}",
                    req.uri(),
                    req.app_data::<Arc<AppConfig>>()
                );
                async move { HttpResponse::Ok().body(body) }
            }));
        });

        server.run(server_config.configure()).await
    })?
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
