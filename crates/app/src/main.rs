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
    let app = App::boot()?;
    app.run()
}

#[cfg(not(feature = "std"))]
#[unsafe(no_mangle)]
fn main(argc: usize, argv: *const *const c_char) -> c_int {
    let app = App::boot(argc, argv).unwrap();
    app.run().unwrap();
    libc::EXIT_SUCCESS
}
