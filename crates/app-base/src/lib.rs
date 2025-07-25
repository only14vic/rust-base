#![cfg_attr(not(feature = "std"), no_std)]

#[allow(unused_imports)]
#[macro_use]
extern crate core;
extern crate alloc;

#[cfg(not(feature = "std"))]
mod no_std;

mod binds;

pub mod ini;
pub mod base;
pub mod log;
pub mod env;
pub mod mem_stats;
pub mod config;
pub mod prelude;
pub mod macros;
pub mod args;
pub mod convert;
pub mod dirs;
pub mod di;
