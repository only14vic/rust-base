#[cfg(not(feature = "std"))]
extern crate libc;

use {
    crate::prelude::Void,
    alloc::{boxed::Box, string::String},
    core::{
        ptr::null_mut,
        sync::atomic::{AtomicPtr, Ordering}
    }
};
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

static ENV: AtomicPtr<Env> = AtomicPtr::new(null_mut());

#[derive(Debug, Clone)]
pub struct Env {
    pub is_test: bool,
    pub is_prod: bool,
    pub is_dev: bool,
    pub is_debug: bool,
    pub is_release: bool,
    pub env: String
}

impl Default for Env {
    fn default() -> Self {
        Self {
            is_test: cfg!(test) || getenv("APP_ENV").map(|v| &v == "test").unwrap_or_default(),
            is_prod: getenv("APP_ENV").map(|v| &v == "prod").unwrap_or_default(),
            is_dev: getenv("APP_ENV").map(|v| &v != "prod").unwrap_or_default(),
            is_debug: getenv("APP_DEBUG").map(|v| &v == "1").unwrap_or_default(),
            is_release: cfg!(debug_assertions) == false,
            env: getenv("APP_ENV").unwrap_or_default()
        }
    }
}

impl Env {
    #[inline]
    pub fn from_static() -> &'static Self {
        let mut env = ENV.load(Ordering::Acquire);

        if env.is_null() {
            env = Box::leak(Box::new(Self::default()));
            if let Err(prev) =
                ENV.compare_exchange(null_mut(), env, Ordering::SeqCst, Ordering::Relaxed)
            {
                drop(unsafe { Box::from_raw(env) });
                env = prev;
            }
        }

        unsafe { &*env }
    }

    #[inline]
    pub fn env<'a>() -> &'a str {
        Self::from_static().env.as_str()
    }

    #[inline]
    pub fn is_test() -> bool {
        Self::from_static().is_test
    }

    #[inline]
    pub fn is_prod() -> bool {
        Self::from_static().is_prod
    }

    #[inline]
    pub fn is_dev() -> bool {
        Self::from_static().is_dev
    }

    #[inline]
    pub fn is_debug() -> bool {
        Self::from_static().is_debug
    }

    #[inline]
    pub fn is_release() -> bool {
        Self::from_static().is_release
    }
}
