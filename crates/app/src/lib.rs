#![cfg_attr(not(feature = "std"), no_std)]

#[allow(unused_imports)]
#[macro_use]
extern crate core;
extern crate alloc;

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

#[cfg(feature = "std")]
mod http_server_module;
#[cfg(feature = "std")]
pub use http_server_module::*;
use app_base::prelude::AppSimpleModule;

#[rustfmt::skip]
use app_base::prelude::{
    App as AppBase,
    AppConfig as AppBaseConfig,
    AppConfigModule,
    AppModule,
    AppModuleExt
};

#[cfg(feature = "std")]
use app_migrator::MigratorModule;

pub type App = AppBase<Config>;
pub type AppConfig = AppBaseConfig<Config>;

pub const MODULE_SIMPLE: AppModule<Config> = AppSimpleModule::handle;
pub const MODULE_CONFIG: AppModule<Config> = AppConfigModule::handle;
#[cfg(feature = "std")]
pub const MODULE_HTTP_SERVER: AppModule<Config> = HttpServerModule::handle;
#[cfg(feature = "std")]
pub const MODULE_MIGRATOR: AppModule<Config> = MigratorModule::handle;
