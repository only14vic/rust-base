#![cfg_attr(not(feature = "std"), no_std)]
#![no_main]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;
extern crate core;

use {
    app_base::prelude::*,
    core::ffi::{c_char, c_int}
};

const CONFIG_FILE_NAME: &str = "app.ini";

#[unsafe(no_mangle)]
fn main(argc: usize, argv: *const *const c_char) -> c_int {
    main_run(argc, argv).inspect_err(|e| panic!("{e}")).ok();
    libc::EXIT_SUCCESS
}

#[allow(unused_variables)]
fn main_run(argc: usize, argv: *const *const c_char) -> Void {
    dotenv(false);
    let mut log = Logger::init()?;

    #[rustfmt::skip]
    let config = app::Config::load(
        CONFIG_FILE_NAME,
        #[cfg(not(feature = "std"))]
        argc,
        #[cfg(not(feature = "std"))]
        argv
    )?;

    log.configure(&config.base.log)?;

    ok()
}
