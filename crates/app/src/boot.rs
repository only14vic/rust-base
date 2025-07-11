use {crate::Config, app_base::prelude::*, core::ffi::c_char};

pub struct Boot;

#[unsafe(no_mangle)]
#[allow(unused_variables)]
pub extern "C" fn boot(argc: usize, argv: *const *const c_char) {
    #[cfg(feature = "std")]
    let _ = Boot::boot().unwrap();
    #[cfg(not(feature = "std"))]
    let _ = Boot::boot(argc, argv).unwrap();
}

impl Boot {
    const CONFIG_FILE_NAME: &str = "app.ini";

    pub fn boot(
        #[cfg(not(feature = "std"))] argc: usize,
        #[cfg(not(feature = "std"))] argv: *const *const c_char
    ) -> Void {
        dotenv(false);
        let mut log = Logger::init()?;

        #[cfg(feature = "std")]
        let args = Config::parse_args()?;
        #[cfg(not(feature = "std"))]
        let args = Config::parse_args(argc, argv)?;

        let config = Config::load(Self::CONFIG_FILE_NAME, &args)?;

        log.configure(&config.base.log)?;

        let di = Di::from_static();
        di.set(log);
        di.set(args);
        di.set(config);

        ok()
    }
}
