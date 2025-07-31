#[cfg(not(feature = "std"))]
use core::panic::PanicInfo;

use {
    crate::AppConfig,
    alloc::{boxed::Box, sync::Arc},
    app_base::prelude::*,
    core::{
        ffi::{c_char, c_int, c_void},
        ops::{Deref, DerefMut}
    }
};

pub struct App {
    di: Di,
    modules: Vec<Box<dyn FnOnce(&mut App) -> Void>>
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
        let global_di = Di::from_static();
        let log = global_di.get::<&mut Logger>();
        let config = self.get::<AppConfig>();

        self.clear();

        if let Some(config) = config
            && config.options.clear_static_di
        {
            global_di.clear();
        }

        log::trace!("App finished");

        if let Some(log) = log
            && let Some(log) = Arc::into_inner(log)
        {
            log.log_close();
        }
    }
}

impl App {
    #[inline]
    pub fn config(&self) -> &AppConfig {
        self.get_ref::<AppConfig>()
            .expect("App container is not initialized")
    }

    pub fn boot(
        #[cfg(not(feature = "std"))] argc: c_int,
        #[cfg(not(feature = "std"))] argv: *const *const c_char
    ) -> Ok<Self> {
        #[cfg(not(feature = "std"))]
        set_panic_handler(Self::panic_handler);

        dotenv(false);
        let log = Logger::init()?;

        #[cfg(feature = "std")]
        let args = AppConfig::parse_args()?;

        #[cfg(not(feature = "std"))]
        let args = AppConfig::parse_args(argc, argv)?;

        let config = AppConfig::load(Some(&args))?;

        log.configure(&config.base.log)?;
        log::trace!("Loaded: {config:#?}");

        let mut di = Di::default();
        di.set(args);
        di.set(config);

        let global_di = Di::from_static();
        global_di.set(log);
        global_di.add(di.get::<AppConfig>().unwrap());

        let app = Self { di, modules: Default::default() };

        Ok(app)
    }

    pub fn register_module(
        &mut self,
        module: impl FnOnce(&mut App) -> Void + 'static
    ) -> &mut Self {
        self.modules.push(Box::new(module));
        self
    }

    pub fn load_modules(&mut self) -> Void {
        let modules = core::mem::take(&mut self.modules);
        for module in modules {
            module(self)?
        }
        ok()
    }

    #[cfg(not(feature = "std"))]
    fn panic_handler(info: &PanicInfo) {
        eprintln!("ERROR: {info}");

        log::error!("{info}");

        let global_di = Di::from_static();
        let log = global_di.get::<&mut Logger>();
        global_di.clear();

        if let Some(log) = log
            && let Some(log) = Arc::into_inner(log)
        {
            log.log_close();
        }
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
    extern "C" fn app_free(app: *mut c_void) {
        let _ = unsafe { Box::from_raw(app.cast::<Self>()) };
    }

    #[unsafe(no_mangle)]
    extern "C" fn app_run(app: *mut c_void) {
        let app = unsafe { &mut *app.cast::<Self>() };
        let _ = app.run().inspect_err(|e| {
            log::error!("{e}");
            let _ = unsafe { Box::from_raw(app) };
            panic!("{e}")
        });
    }
}
