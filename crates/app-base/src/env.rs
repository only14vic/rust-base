#[cfg(not(feature = "std"))]
extern crate libc;

use {crate::prelude::Ok, alloc::string::String};
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
    fn load_env(&mut self) -> Ok<&mut Self>;
}

pub struct Env;

impl Env {
    #[inline]
    pub fn is_test() -> bool {
        #[cfg(feature = "std")]
        return cfg!(test) || std::env::var("APP_ENV").unwrap_or_default() == "test";
        #[cfg(not(feature = "std"))]
        return cfg!(test);
    }

    #[inline]
    pub fn is_prod() -> bool {
        #[cfg(feature = "std")]
        return Self::is_release()
            || std::env::var("APP_ENV").unwrap_or_default() == "prod";
        #[cfg(not(feature = "std"))]
        return Self::is_release();
    }

    #[inline]
    pub fn is_dev() -> bool {
        Self::is_prod() == false
    }

    #[inline]
    pub fn is_debug() -> bool {
        cfg!(debug_assertions) == true
    }

    #[inline]
    pub fn is_release() -> bool {
        cfg!(debug_assertions) == false
    }
}
