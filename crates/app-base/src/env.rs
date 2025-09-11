use {
    alloc::{boxed::Box, string::String},
    core::{
        ptr::null_mut,
        sync::atomic::{AtomicBool, AtomicPtr, Ordering}
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
    fn load_env(&mut self);
}

static ENV: AtomicPtr<Env> = AtomicPtr::new(null_mut());
static LOCK: AtomicBool = AtomicBool::new(false);

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
            is_prod: getenv("APP_ENV")
                .map(|v| &v.to_lowercase() == "prod" || v.is_empty())
                .unwrap_or(true),
            is_dev: getenv("APP_ENV")
                .map(|v| &v.to_lowercase() != "prod" && v.is_empty() == false)
                .unwrap_or(false),
            is_debug: getenv("APP_DEBUG")
                .map(|v| ["1", "true", "on"].contains(&v.to_lowercase().as_str()))
                .unwrap_or_default(),
            is_release: cfg!(debug_assertions) == false,
            env: getenv("APP_ENV")
                .map(|v| v.to_lowercase())
                .unwrap_or_default()
        }
    }
}

impl Env {
    #[inline]
    pub fn from_static() -> &'static mut Self {
        let mut env = ENV.load(Ordering::Acquire);

        if env.is_null() {
            if LOCK.swap(true, Ordering::SeqCst) == false {
                env = Box::leak(Box::new(Self::default()));
                ENV.store(env, Ordering::Release);
            } else {
                loop {
                    env = ENV.load(Ordering::Acquire);
                    if env.is_null() == false {
                        break;
                    }
                }
            }
        }

        unsafe { &mut *env }
    }

    #[inline]
    pub fn env() -> &'static str {
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
