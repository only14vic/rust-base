#![cfg_attr(not(feature = "std"), no_std)]

#[allow(unused_imports)]
#[macro_use]
extern crate core;
extern crate alloc;

#[cfg(not(feature = "std"))]
pub use libc_print::std_name::*;

mod binds;
mod ini;

mod base;
mod logger;

pub use {base::*, ini::*, logger::*, set_from_iter_derive::*};
