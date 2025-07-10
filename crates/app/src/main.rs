#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;
extern crate core;

#[cfg(not(feature = "std"))]
use core::ffi::{c_char, c_int};

use {app::Config, app_base::prelude::*};

const CONFIG_FILE_NAME: &str = "app.ini";

#[cfg(feature = "std")]
fn main() -> Void {
    run()
}

#[cfg(not(feature = "std"))]
#[unsafe(no_mangle)]
fn main(argc: usize, argv: *const *const c_char) -> c_int {
    run(argc, argv).inspect_err(|e| panic!("{e}")).ok();
    libc::EXIT_SUCCESS
}

fn run(
    #[cfg(not(feature = "std"))] argc: usize,
    #[cfg(not(feature = "std"))] argv: *const *const c_char
) -> Void {
    dotenv(false);
    let log = Logger::init()?;

    let di = Di::from_static();
    di.set(log);

    #[rustfmt::skip]
    let config = Config::load(
        CONFIG_FILE_NAME,
        #[cfg(not(feature = "std"))]
        argc,
        #[cfg(not(feature = "std"))]
        argv
    )?;
    di.set(config);

    ok()
}
