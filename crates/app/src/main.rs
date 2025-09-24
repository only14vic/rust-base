#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;
extern crate core;

#[cfg(not(feature = "std"))]
use core::ffi::{c_char, c_int};

#[allow(unused_imports)]
use {app::App, app::*, app_base::prelude::*};

#[cfg(feature = "std")]
fn main() -> Void {
    #[rustfmt::skip]
    App::new([
            MODULE_MAIN,
            MODULE_CONFIG,
            #[cfg(feature="web")]
            MODULE_WEB,
            #[cfg(feature="migrator")]
            MODULE_MIGRATOR,
            #[cfg(feature="desktop")]
            MODULE_DESKTOP,
        ])
        .boot()
        .inspect_err(|e| log::error!("{e}"))?
        .run()
        .inspect_err(|e| log::error!("{e}"))?;

    ok()
}

#[cfg(not(feature = "std"))]
#[unsafe(no_mangle)]
fn main(argc: c_int, argv: *const *const c_char) -> c_int {
    #[rustfmt::skip]
    let mut app = App::new([
        MODULE_MAIN,
        MODULE_CONFIG,
    ]);

    if let Err(e) = app.boot(argc, argv) {
        log::error!("{e}");
        eprintln!("{e}");
        return libc::EXIT_FAILURE;
    }

    if let Err(e) = app.run() {
        log::error!("{e}");
        eprintln!("{e}");
        return libc::EXIT_FAILURE;
    }

    libc::EXIT_SUCCESS
}
