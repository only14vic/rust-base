#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;
extern crate core;

#[cfg(not(feature = "std"))]
use core::ffi::{c_char, c_int};

use {app::*, app_base::prelude::*};

#[cfg(feature = "std")]
fn main() -> Void {
    run()
}

#[cfg(not(feature = "std"))]
#[unsafe(no_mangle)]
fn main(argc: usize, argv: *const *const c_char) -> c_int {
    let _ = run(argc, argv).unwrap();
    libc::EXIT_SUCCESS
}

pub fn run(
    #[cfg(not(feature = "std"))] argc: usize,
    #[cfg(not(feature = "std"))] argv: *const *const c_char
) -> Void {
    #[cfg(feature = "std")]
    Boot::boot()?;
    #[cfg(not(feature = "std"))]
    Boot::boot(argc, argv)?;

    ok()
}
