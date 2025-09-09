#[cfg(feature = "std")]
use std::panic::PanicHookInfo;
#[cfg(not(feature = "std"))]
use core::panic::PanicInfo;

use {
    super::AppConfig,
    crate::{app::AppConfigExt, prelude::*},
    alloc::{boxed::Box, format, sync::Arc, vec::Vec},
    core::{
        ffi::{CStr, c_char, c_int, c_uint},
        mem::forget,
        ops::{Deref, DerefMut},
        ptr::addr_eq
    }
};

pub type AppModule<C> = fn(&mut App<C>, AppEvent) -> Void;

#[derive(Default)]
pub struct App<C>
where
    C: AppConfigExt
{
    di: Di,
    config: Arc<AppConfig<C>>,
    modules: Vec<AppModule<C>>,
    commands: IndexMap<&'static str, AppModule<C>>,
    pub clear_global: bool
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AppEvent {
    APP_INIT,
    APP_LOADED,
    APP_BOOT,
    APP_RUN,
    APP_END
}

impl<C> Deref for App<C>
where
    C: AppConfigExt
{
    type Target = Di;

    fn deref(&self) -> &Self::Target {
        &self.di
    }
}

impl<C> DerefMut for App<C>
where
    C: AppConfigExt
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.di
    }
}

impl<C> Drop for App<C>
where
    C: AppConfigExt
{
    fn drop(&mut self) {
        let _ = self.trigger_event(AppEvent::APP_END);

        core::mem::take(&mut self.di);
        core::mem::take(&mut self.commands);
        core::mem::forget(core::mem::take(&mut self.modules));

        let global_di = Di::from_static();

        if global_di
            .get_ref::<Box<Self>>()
            .map(|app| addr_eq(app.deref(), self))
            == Some(true)
        {
            forget(global_di.remove::<Box<Self>>().unwrap());
        }

        if self.clear_global {
            global_di.clear();
        }

        Env::is_debug().then(|| log::trace!("App finished"));

        Logger::from_static().unwrap().log_close();
    }
}

impl<C> App<C>
where
    C: AppConfigExt
{
    pub fn new(modules: impl IntoIterator<Item = AppModule<C>>) -> Self {
        Self {
            di: Di::default(),
            config: Arc::new(AppConfig::<C>::default()),
            modules: Vec::from_iter(modules),
            commands: Default::default(),
            clear_global: true
        }
    }

    #[inline]
    pub fn config(&self) -> &Arc<AppConfig<C>> {
        &self.config
    }

    pub fn boot(
        &mut self,
        #[cfg(not(feature = "std"))] argc: c_int,
        #[cfg(not(feature = "std"))] argv: *const *const c_char
    ) -> Ok<&mut Self> {
        #[cfg(feature = "std")]
        std::panic::set_hook(Box::new(Self::panic_handler));
        #[cfg(not(feature = "std"))]
        set_panic_handler(Box::new(Self::panic_handler));

        dotenv(false);

        let log = Logger::from_static().unwrap();

        let mut args = Args::new([
            ("exe", &["0"][..], None),
            ("command", &["1"], Some(AppConfig::<C>::DEFAULT_COMMAND)),
            ("help", &["-h"], None)
        ])
        .unwrap();

        //
        // Preloading of command line arguments.
        // Skips undefined arguments for preloading.
        //
        args.set_undefined(ArgUndefined::Skip);
        #[cfg(feature = "std")]
        args.parse_args(std::env::args().collect())?;
        #[cfg(not(feature = "std"))]
        unsafe {
            args.parse_argc(argc, argv)?
        };
        // Throws error if undefined arguments are detected for next load.
        args.set_undefined(ArgUndefined::Error);
        Env::is_debug().then(|| {
            log::trace!(
                "Preloaded command line arguments: {:?}",
                &args
                    .args
                    .iter()
                    .filter(|(_, v)| v.is_some())
                    .collect::<Vec<_>>()
            )
        });

        self.set(args);

        self.trigger_event(AppEvent::APP_INIT)?;

        //
        // Full loading of command line arguments after initializing modules.
        // Modules can add arguments depending on the command.
        //
        let args = self.get_mut::<Args>().unwrap().unwrap();
        // Skips undefined arguments on tests.
        if Env::is_test() {
            args.set_undefined(ArgUndefined::Skip);
        }
        #[cfg(feature = "std")]
        args.parse_args(std::env::args().collect())?;
        #[cfg(not(feature = "std"))]
        unsafe {
            args.parse_argc(argc, argv)?
        };
        Env::is_debug().then(|| {
            log::trace!(
                "Loaded command line arguments: {:?}",
                &args
                    .args
                    .iter()
                    .filter(|(_, v)| v.is_some())
                    .collect::<Vec<_>>()
            )
        });

        let args = self.get::<Args>().unwrap();
        self.config.try_mut().unwrap().load(Some(&args))?;

        log.configure(&self.config.base.log)?;

        Env::is_debug().then(|| log::trace!("Loaded: {:#?}", &self.config));

        self.trigger_event(AppEvent::APP_LOADED)?;
        self.trigger_event(AppEvent::APP_BOOT)?;

        Ok(self)
    }

    pub fn register_command(&mut self, command: &'static str, module: AppModule<C>) -> &mut Self {
        self.commands.insert(command, module);
        self
    }

    pub fn register_module(&mut self, module: AppModule<C>) -> &mut Self {
        self.modules.push(module);
        self
    }

    fn trigger_event(&mut self, event: AppEvent) -> Void {
        Env::is_debug().then(|| log::trace!("Triggering event: {event:#?}"));

        for module in self.modules.clone() {
            module(self, event)?;
        }

        ok()
    }

    pub fn run(&mut self) -> Void {
        let args = self.get_ref::<Args>().unwrap();
        let command = args
            .get("command")
            .ok_or("Command line argument 'command' is undefined")?
            .as_ref()
            .ok_or("Command not specified")?;

        if let Some(module) = self
            .commands
            .iter()
            .find_map(|(name, module)| name.eq(&command).then_some(module))
        {
            Env::is_debug().then(|| log::trace!("Triggering event: {:#?}", AppEvent::APP_RUN));
            module(self, AppEvent::APP_RUN)
        } else if command == AppConfig::<C>::DEFAULT_COMMAND
            && self.commands.is_empty()
            && let Some(module) = self.modules.first()
        {
            Env::is_debug().then(|| log::trace!("Triggering event: {:#?}", AppEvent::APP_RUN));
            module(self, AppEvent::APP_RUN)
        } else {
            Err(format!("Invalid command '{command}'"))?
        }
    }

    #[cfg(feature = "std")]
    fn panic_handler(info: &PanicHookInfo) {
        eprintln!("PANIC: {info}");
        log::error!("{info}");
    }

    #[cfg(not(feature = "std"))]
    fn panic_handler(info: &PanicInfo) {
        eprintln!("PANIC: {info}");
        log::error!("{info}");
        Di::from_static().clear();
        Logger::from_static().unwrap().log_close();
    }

    #[unsafe(no_mangle)]
    extern "C" fn app_new(modules: *mut AppModule<C>, count: c_uint) -> *mut App<C> {
        let modules = unsafe { Vec::from_raw_parts(modules, count as usize, count as usize) };
        let app = Self::new(modules);
        Box::into_raw(app.into())
    }

    #[unsafe(no_mangle)]
    #[allow(unused_variables)]
    extern "C" fn app_boot(app: *mut App<C>, argc: c_int, argv: *const *const c_char) {
        #[cfg(not(feature = "std"))]
        Di::from_static().set(unsafe { Box::from_raw(app) });

        let app = unsafe { &mut *app };

        #[cfg(feature = "std")]
        let _ = app.boot().unwrap_or_else(|e| panic!("{e}"));

        #[cfg(not(feature = "std"))]
        let _ = app.boot(argc, argv).unwrap_or_else(|e| panic!("{e}"));
    }

    #[unsafe(no_mangle)]
    extern "C" fn app_run(app: *mut App<C>) {
        let app = unsafe { &mut *app };
        app.run().unwrap_or_else(|e| panic!("{e}"));
    }

    #[unsafe(no_mangle)]
    extern "C" fn app_free(app: *mut App<C>) {
        let _ = unsafe { Box::from_raw(app) };
    }

    #[unsafe(no_mangle)]
    #[allow(improper_ctypes_definitions)]
    unsafe extern "C" fn app_register_command(
        app: *mut App<C>,
        command: *const c_char,
        module: AppModule<C>
    ) {
        unsafe {
            let app = &mut *app;
            let command = CStr::from_ptr(command).to_str().unwrap();
            app.register_command(command, module);
        }
    }
}
