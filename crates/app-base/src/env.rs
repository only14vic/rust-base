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
