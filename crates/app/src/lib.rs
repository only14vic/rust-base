#![cfg_attr(not(feature = "std"), no_std)]

#[allow(unused_imports)]
#[macro_use]
extern crate core;
extern crate alloc;

mod app_module;

pub use app_module::*;

#[cfg(feature = "std")]
mod config;
#[cfg(feature = "std")]
pub use config::*;

#[cfg(not(feature = "std"))]
mod config_no_std;
#[cfg(not(feature = "std"))]
pub use config_no_std::*;

#[cfg(feature = "std")]
mod http_server;
#[cfg(feature = "std")]
pub use http_server::*;
