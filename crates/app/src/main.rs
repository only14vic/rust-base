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

const MODULE_APP: fn(&mut App, event: AppEvent) -> Void = module_handler;

#[cfg(feature = "std")]
fn main() -> Void {
    let mut app = App::new([MODULE_APP]);
    app.boot().inspect_err(|e| log::error!("{e}"))?;
    app.run().inspect_err(|e| log::error!("{e}"))
}

#[cfg(not(feature = "std"))]
#[unsafe(no_mangle)]
fn main(argc: c_int, argv: *const *const c_char) -> c_int {
    let mut app = App::new([MODULE_APP]);
    let _ = app.boot(argc, argv).inspect_err(|e| panic!("{e}"));
    let _ = app.run().inspect_err(|e| panic!("{e}"));

    libc::EXIT_SUCCESS
}

fn module_handler(app: &mut App, event: AppEvent) -> Void {
    match event {
        AppEvent::APP_INIT => {
            app.register_command(AppConfig::DEFAULT_COMMAND, MODULE_APP);
            ok()
        },
        #[cfg(not(feature = "std"))]
        AppEvent::APP_RUN => {
            mem_stats();
            ok()
        },
        #[cfg(feature = "std")]
        AppEvent::APP_RUN => {
            use {
                actix_web::{HttpRequest, HttpResponse, web},
                app_async::actix_with_tokio_start,
                app_web::HttpServer,
                std::sync::Arc
            };

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

                ok()
            })?
        },
        _ => ok()
    }
}
