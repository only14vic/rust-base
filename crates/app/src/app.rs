use {
    crate::AppConfig,
    alloc::boxed::Box,
    app_base::prelude::*,
    core::{
        ffi::{c_char, c_int, c_void},
        ops::{Deref, DerefMut}
    }
};

pub struct App {
    di: Di
}

impl Deref for App {
    type Target = Di;

    fn deref(&self) -> &Self::Target {
        &self.di
    }
}

impl DerefMut for App {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.di
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if self.config().options.clear_static_di {
            Di::from_static().clear();
            log::trace!("Static Di cleared");
        }

        log::trace!("App finished");
    }
}

impl App {
    const CONFIG_FILE_NAME: &str = "app.ini";

    #[inline]
    pub fn config(&self) -> &AppConfig {
        self.get_ref::<AppConfig>()
            .expect("App container is not initialized")
    }

    pub fn boot(
        #[cfg(not(feature = "std"))] argc: c_int,
        #[cfg(not(feature = "std"))] argv: *const *const c_char
    ) -> Ok<Self> {
        dotenv(false);
        let log = Logger::init()?;

        #[cfg(feature = "std")]
        let args = AppConfig::parse_args()?;

        #[cfg(not(feature = "std"))]
        let args = AppConfig::parse_args(argc, argv)?;

        let config = AppConfig::load(Self::CONFIG_FILE_NAME, Some(&args))?;

        log.configure(&config.base.log)?;
        log::trace!("Loaded: {config:#?}");

        let mut di = Di::default();
        di.set(args);
        di.set(config);

        let app = Self { di };

        Ok(app)
    }

    pub fn run(&mut self) -> Void {
        mem_stats();
        ok()
    }

    #[unsafe(no_mangle)]
    #[allow(unused_variables)]
    extern "C" fn app_boot(argc: c_int, argv: *const *const c_char) -> *mut c_void {
        #[rustfmt::skip]
        #[cfg(feature = "std")]
        let app = Self::boot()
            .inspect_err(|e| panic!("{e}"))
            .unwrap();

        #[cfg(not(feature = "std"))]
        let app = Self::boot(argc, argv)
            .inspect_err(|e| panic!("{e}"))
            .unwrap();

        Box::into_raw(app.into()).cast()
    }

    #[unsafe(no_mangle)]
    extern "C" fn app_finish(app: *mut c_void) {
        let _ = unsafe { Box::from_raw(app.cast::<Self>()) };
    }

    #[unsafe(no_mangle)]
    extern "C" fn app_run(app: *mut c_void) {
        let app = unsafe { &mut *app.cast::<Self>() };
        let _ = app.run().inspect_err(|e| panic!("{e}"));
    }
}
