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
    App::new([MODULE_APP, MODULE_APP_CONFIG])
        .boot()
        .inspect_err(|e| log::error!("{e}"))?
        .run()
        .inspect_err(|e| log::error!("{e}"))
}

#[cfg(not(feature = "std"))]
#[unsafe(no_mangle)]
fn main(argc: c_int, argv: *const *const c_char) -> c_int {
    App::new([MODULE_APP, MODULE_APP_CONFIG])
        .boot(argc, argv)
        .unwrap_or_else(|e| panic!("{e}"))
        .run()
        .unwrap_or_else(|e| panic!("{e}"));

    libc::EXIT_SUCCESS
}
