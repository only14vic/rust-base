mod actix_config;
mod web_config;

pub mod middleware;
pub mod ext;
pub mod api;

#[allow(unused)]
pub use {actix_config::*, web_config::*};
