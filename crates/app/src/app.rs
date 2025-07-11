use {
    crate::AppConfig,
    alloc::boxed::Box,
    app_base::prelude::*,
    core::{
        ffi::{c_char, c_void},
        ops::{Deref, DerefMut},
        pin::Pin
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
        let _log = if self.config().options.clear_static_di {
            let di = Di::from_static();
            let log = di.get::<Pin<Box<Logger>>>();
            Di::from_static().clear();
            log::trace!("Static Di cleared");
            log
        } else {
            None
        };

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
        #[cfg(not(feature = "std"))] argc: usize,
        #[cfg(not(feature = "std"))] argv: *const *const c_char
    ) -> Ok<Self> {
        dotenv(false);
        let mut log = Logger::init()?;

        #[cfg(feature = "std")]
        let args = AppConfig::parse_args()?;
        #[cfg(not(feature = "std"))]
        let args = AppConfig::parse_args(argc, argv)?;

        let config = AppConfig::load(Self::CONFIG_FILE_NAME, Some(&args))?;

        log.configure(&config.base.log)?;
        log::trace!("Loaded: {config:#?}");

        let mut di = Di::default();
        di.set(log);
        di.set(args);
        di.set(config);

        let app = Self { di };

        Ok(app)
    }

    pub fn run(self) -> Void {
        mem_stats();
        ok()
    }
}

#[unsafe(no_mangle)]
#[allow(unused_variables)]
pub extern "C" fn app_start(argc: usize, argv: *const *const c_char) -> *const c_void {
    #[cfg(feature = "std")]
    let app = App::boot().unwrap();
    #[cfg(not(feature = "std"))]
    let app = App::boot(argc, argv).unwrap();

    Box::into_raw(app.into()).cast()
}
