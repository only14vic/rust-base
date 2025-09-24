#![cfg_attr(not(feature = "std"), no_std)]

#[allow(unused_imports)]
#[macro_use]
extern crate core;
extern crate alloc;

#[rustfmt::skip]
use app_base::prelude::{
    App as AppBase,
    AppConfig as AppBaseConfig,
    AppConfigModule,
    AppModule,
    AppModuleExt
};

#[cfg(not(feature = "std"))]
mod config_no_std;
#[cfg(not(feature = "std"))]
pub use config_no_std::*;

#[cfg(feature = "std")]
mod config;
#[cfg(feature = "std")]
pub use config::*;

pub type App = AppBase<Config>;
pub type AppConfig = AppBaseConfig<Config>;

pub static MODULE_CONFIG: AppModule<Config> = AppConfigModule::handle;

use app_base::prelude::AppSimpleModule;
pub static MODULE_SIMPLE: AppModule<Config> = AppSimpleModule::handle;

mod main_module;
pub use main_module::*;
pub static MODULE_MAIN: AppModule<Config> = MainModule::handle;

#[cfg(feature = "web")]
use app_web::WebModule;
#[cfg(feature = "web")]
pub static MODULE_WEB: AppModule<Config> = WebModule::handle;

#[cfg(feature = "migrator")]
use app_migrator::MigratorModule;
#[cfg(feature = "migrator")]
pub static MODULE_MIGRATOR: AppModule<Config> = MigratorModule::handle;

#[cfg(feature = "desktop")]
use app_desktop::DesktopModule;
#[cfg(feature = "desktop")]
pub static MODULE_DESKTOP: AppModule<Config> = DesktopModule::handle;
