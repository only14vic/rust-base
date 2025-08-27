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
    let res = std::panic::catch_unwind(|| {
        App::new([MODULE_APP, MODULE_APP_CONFIG])
            .boot()
            .inspect_err(|e| log::error!("{e}"))?
            .run()
            .inspect_err(|e| log::error!("{e}"))
    });

    let log_closer = Logger::from_static().unwrap().get_closer();

    match res {
        // if panic
        Err(e) => {
            use std::process::abort;

            let e = if let Some(e) = e.downcast_ref::<Err>() {
                e.to_string()
            } else if let Some(e) = e.downcast_ref::<String>() {
                e.to_string()
            } else {
                "Panic occures".into()
            };

            log::error!("{e}");
            drop(log_closer);

            abort();
        },
        Ok(Err(e)) => Err(e)?,
        Ok(res) => res
    }
}

#[cfg(not(feature = "std"))]
#[unsafe(no_mangle)]
fn main(argc: c_int, argv: *const *const c_char) -> c_int {
    App::new([MODULE_APP, MODULE_APP_CONFIG])
        .boot(argc, argv)
        .unwrap_or_else(|e| panic!("{e}"))
        .run()
        .unwrap_or_else(|e| panic!("{e}"));

    Logger::from_static().unwrap().log_close();

    libc::EXIT_SUCCESS
}
