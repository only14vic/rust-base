#![cfg_attr(not(feature = "std"), no_std)]

#[allow(unused_imports)]
#[macro_use]
extern crate core;
extern crate alloc;

mod app;
mod app_module;
mod app_config;
mod app_config_module;

pub use {app::*, app_config::*, app_config_module::*, app_module::*};

#[cfg(feature = "std")]
mod http_server;
#[cfg(feature = "std")]
pub use http_server::*;
