use std::env;

pub struct Env;

impl Env {
    #[inline]
    pub fn is_test() -> bool {
        cfg!(test) || env::var("APP_ENV").unwrap_or_default() == "test"
    }

    #[inline]
    pub fn is_prod() -> bool {
        Self::is_release() || env::var("APP_ENV").unwrap_or_default() == "prod"
    }

    #[inline]
    pub fn is_dev() -> bool {
        Self::is_prod() == false
    }

    #[inline]
    pub fn is_debug() -> bool {
        env::var("APP_DEBUG").unwrap_or_default() == "1"
    }

    #[inline]
    pub fn is_release() -> bool {
        cfg!(debug_assertions) == false
    }
}
