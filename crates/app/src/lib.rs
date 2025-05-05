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
mod main_module;
#[cfg(feature = "std")]
pub use main_module::*;
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
use {app_migrator::MigratorModule, app_web::WebModule};

pub type App = AppBase<Config>;
pub type AppConfig = AppBaseConfig<Config>;

pub static MODULE_SIMPLE: AppModule<Config> = AppSimpleModule::handle;
pub static MODULE_CONFIG: AppModule<Config> = AppConfigModule::handle;
#[cfg(feature = "std")]
pub static MODULE_MAIN: AppModule<Config> = MainModule::handle;
#[cfg(feature = "std")]
pub static MODULE_WEB: AppModule<Config> = WebModule::handle;
#[cfg(feature = "std")]
pub static MODULE_MIGRATOR: AppModule<Config> = MigratorModule::handle;
