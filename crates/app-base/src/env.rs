#[cfg(not(feature = "std"))]
extern crate libc;

use {crate::prelude::Void, alloc::string::String};
#[cfg(not(feature = "std"))]
use {alloc::ffi::CString, alloc::string::ToString, core::ffi::CStr, core::str::FromStr};

pub fn getenv(name: &str) -> Option<String> {
    #[cfg(feature = "std")]
    return std::env::var(name).ok();

    #[cfg(not(feature = "std"))]
    unsafe {
        if let Ok(cstr) = CString::from_str(name) {
            match libc::getenv(cstr.as_ptr()) {
                ptr if ptr.is_null() == false => {
                    match CStr::from_ptr(ptr).to_str() {
                        Ok(str) => str.to_string().into(),
                        _ => None
                    }
                },
                _ => None
            }
        } else {
            None
        }
    }
}

pub trait LoadEnv {
    fn load_env(&mut self) -> Void;
}

thread_local! {
    static ENV: Env = Env::default();
}

#[derive(Debug, Clone)]
pub struct Env {
    pub is_test: bool,
    pub is_prod: bool,
    pub is_dev: bool,
    pub is_debug: bool,
    pub is_release: bool,
    pub env: &'static str
}

impl Default for Env {
    fn default() -> Self {
        Self {
            is_test: cfg!(test) || getenv("APP_ENV").map(|v| &v == "test").unwrap_or_default(),
            is_prod: getenv("APP_ENV").map(|v| &v == "prod").unwrap_or_default(),
            is_dev: getenv("APP_ENV").map(|v| &v != "prod").unwrap_or_default(),
            is_debug: getenv("APP_DEBUG").map(|v| &v == "1").unwrap_or_default(),
            is_release: cfg!(debug_assertions) == false,
            env: String::leak(getenv("APP_ENV").unwrap_or_default())
        }
    }
}

impl Env {
    #[inline]
    pub fn env() -> &'static str {
        ENV.with(|e| e.env)
    }

    #[inline]
    pub fn is_test() -> bool {
        ENV.with(|e| e.is_test)
    }

    #[inline]
    pub fn is_prod() -> bool {
        ENV.with(|e| e.is_prod)
    }

    #[inline]
    pub fn is_dev() -> bool {
        ENV.with(|e| e.is_dev)
    }

    #[inline]
    pub fn is_debug() -> bool {
        ENV.with(|e| e.is_debug)
    }

    #[inline]
    pub fn is_release() -> bool {
        ENV.with(|e| e.is_release)
    }
}
