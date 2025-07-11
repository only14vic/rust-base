use {
    crate::Config,
    alloc::boxed::Box,
    app_base::prelude::*,
    core::{
        ffi::{c_char, c_void},
        pin::Pin
    }
};

pub struct App;

impl Drop for App {
    fn drop(&mut self) {
        let di = Di::from_static();
        let _log = di.get::<Pin<Box<Logger>>>();

        Di::from_static().clear();

        log::trace!("App finished");
    }
}

impl App {
    const CONFIG_FILE_NAME: &str = "app.ini";

    pub fn boot(
        #[cfg(not(feature = "std"))] argc: usize,
        #[cfg(not(feature = "std"))] argv: *const *const c_char
    ) -> Ok<Self> {
        dotenv(false);
        let mut log = Logger::init()?;

        #[cfg(feature = "std")]
        let args = Config::parse_args()?;
        #[cfg(not(feature = "std"))]
        let args = Config::parse_args(argc, argv)?;

        let config = Config::load(Self::CONFIG_FILE_NAME, Some(&args))?;

        log.configure(&config.base.log)?;
        log::trace!("Loaded: {config:#?}");

        let di = Di::from_static();
        di.set(log);
        di.set(args);
        di.set(config);

        let app = Self {};

        Ok(app)
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
