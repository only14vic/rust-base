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

pub fn setenv(name: &str, value: &str) {
    #[cfg(feature = "std")]
    unsafe {
        return std::env::set_var(name, value);
    };

    #[cfg(not(feature = "std"))]
    unsafe {
        let name = CString::new(name).unwrap();
        let value = CString::new(value).unwrap();
        libc::setenv(name.as_ptr(), value.as_ptr(), 1);
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
        let mut env: String = getenv("APP_ENV")
            .map(|v| if v.is_empty() { "prod".into() } else { v })
            .unwrap_or("prod".into());

        if option_env!("APP_ENV") == Some("test") {
            setenv("APP_ENV", "test");
            env.clear();
            env.push_str("test");
        }

        let is_debug = getenv("APP_DEBUG")
            .map(|v| ["1", "true", "on"].contains(&v.to_lowercase().as_str()))
            .unwrap_or(false);

        Self {
            is_test: &env == "test",
            is_prod: &env == "prod",
            is_dev: &env != "prod",
            is_debug,
            is_release: cfg!(debug_assertions) == false,
            env
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
    pub fn reset() {
        Self::from_static().clone_from(&Self::default());
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
