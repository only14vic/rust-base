#[cfg(feature = "std")]
use std::panic::PanicHookInfo;
#[cfg(not(feature = "std"))]
use core::panic::PanicInfo;

#[cfg(not(feature = "std"))]
use {core::ffi::c_char, core::ffi::c_int};
use {
    super::AppConfig,
    crate::{app::AppConfigExt, prelude::*},
    alloc::{boxed::Box, format, string::ToString, sync::Arc, vec::Vec},
    core::{
        mem::forget,
        ops::{Deref, DerefMut},
        ptr::{addr_eq, fn_addr_eq}
    },
    log::set_max_level
};

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AppEvent {
    APP_PRE_INIT,
    APP_INIT,
    APP_BOOT,
    APP_SETUP,
    APP_RUN,
    APP_END
}

#[derive(Default)]
pub struct App<C>
where
    C: AppConfigExt
{
    di: Di,
    args: Args,
    config: Arc<AppConfig<C>>,
    modules: IndexSet<AppModule<C>>,
    commands: IndexMap<&'static str, AppModule<C>>,
    pub clear_global: bool
}

impl<C> Deref for App<C>
where
    C: AppConfigExt
{
    type Target = Di;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.di
    }
}

impl<C> DerefMut for App<C>
where
    C: AppConfigExt
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.di
    }
}

impl<C> AsRef<AppConfig<C>> for App<C>
where
    C: AppConfigExt
{
    #[inline]
    fn as_ref(&self) -> &AppConfig<C> {
        &self.config
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
        core::mem::take(&mut self.modules);

        let global_di = unsafe { Di::from_static_mut() };

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

        Env::is_debug().then(|| log::debug!("App finished"));

        unsafe { Logger::from_static_mut().log_close() };
    }
}

impl<C> App<C>
where
    C: AppConfigExt
{
    pub fn new(modules: impl IntoIterator<Item = AppModule<C>>) -> Self {
        let mut app = Self {
            di: Default::default(),
            args: Args::new([
                ("exe", "0".into(), None),
                ("command", "1".into(), Some(C::COMMAND)),
                ("env-file", None, None),
                ("debug:b", None, None),
                ("version:b", None, None),
                ("help:b", "-h".into(), None)
            ])
            .unwrap(),
            config: Arc::new(AppConfig::<C>::default()),
            modules: Default::default(),
            commands: Default::default(),
            clear_global: true
        };

        for module in modules {
            app.register_module(module);
        }

        app
    }

    #[inline]
    pub fn config(&self) -> &Arc<AppConfig<C>> {
        &self.config
    }

    #[inline]
    pub fn config_mut(&mut self) -> Ok<&mut AppConfig<C>> {
        self.config.try_mut()
    }

    #[inline]
    pub unsafe fn config_static(&self) -> &'static Arc<AppConfig<C>> {
        unsafe { &*(&self.config as *const _) }
    }

    #[inline]
    pub fn args(&self) -> &Args {
        &self.args
    }

    #[inline]
    pub fn args_mut(&mut self) -> &mut Args {
        &mut self.args
    }

    pub fn with_args<'a>(
        &mut self,
        args: impl IntoIterator<Item = (&'a str, &'a str)>
    ) -> &mut Self {
        self.args.extend(
            args.into_iter()
                .map(|(n, v)| (n.into(), v.to_string().into()))
        );
        self
    }

    pub fn boot(
        &mut self,
        #[cfg(not(feature = "std"))] argc: c_int,
        #[cfg(not(feature = "std"))] argv: *const *const c_char
    ) -> Ok<&mut Self> {
        dotenv(false);
        let log = log_init();
        let args = &mut self.args;

        #[cfg(feature = "std")]
        std::panic::set_hook(Box::new(Self::panic_handler));
        #[cfg(not(feature = "std"))]
        set_panic_handler(Box::new(Self::panic_handler));

        //
        // Preloading of command line arguments.
        // Skips undefined arguments for preloading.
        //
        args.set_undefined(ArgUndef::Skip);
        #[cfg(feature = "std")]
        args.parse_args(std::env::args().collect())?;
        #[cfg(not(feature = "std"))]
        unsafe {
            args.parse_argc(argc, argv)?
        };
        // Throws error if undefined arguments are detected for next load.
        args.set_undefined(ArgUndef::Error);

        if let Some(env_file) = args.get("env-file").unwrap() {
            Ini::setenv_from_file(&env_file, true)?;
        }

        if args.get("debug").unwrap() == Some("1") {
            setenv("APP_DEBUG", "1");
            if log::max_level() < log::LevelFilter::Debug {
                set_max_level(log::LevelFilter::Debug);
            }
        }

        // Enables new env variables
        unsafe { Env::reset() };

        Env::is_debug().then(|| {
            log::debug!(
                "Preinit Args: {}",
                &args
                    .iter()
                    .filter_map(|(n, v)| v.as_ref().map(|v| format!("{n}={v}")))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        });

        self.trigger_event(AppEvent::APP_PRE_INIT)?;

        // Correct command name only after registration commands
        self.correct_command_name()?;
        let args = &mut self.args;
        self.config.try_mut().unwrap().init_args(args);

        self.trigger_event(AppEvent::APP_INIT)?;

        //
        // Full loading of command line arguments after initializing modules.
        // Modules can add arguments depending on the command.
        //
        let args = &mut self.args;
        // Skips undefined arguments on tests.
        if Env::is_test() {
            args.set_undefined(ArgUndef::Skip);
        }
        #[cfg(feature = "std")]
        args.parse_args(std::env::args().collect())?;
        #[cfg(not(feature = "std"))]
        unsafe {
            args.parse_argc(argc, argv)?
        };
        // Correct command name again after parse args
        self.correct_command_name()?;
        let args = &mut self.args;

        Env::is_debug().then(|| {
            log::debug!(
                "Loaded Args: {}",
                &args
                    .iter()
                    .filter_map(|(n, v)| v.as_ref().map(|v| format!("{n}={v}")))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        });

        self.config.try_mut().unwrap().load(Some(args))?;

        log.configure(&self.config.base.log)?;

        Env::is_debug().then(|| log::debug!("Loaded {:#?}", &self.config));

        self.trigger_event(AppEvent::APP_BOOT)?;
        self.trigger_event(AppEvent::APP_SETUP)?;

        Ok(self)
    }

    pub fn run(&mut self) -> Void {
        let command = self.command()?;
        let module = self.get_module_by_command(command)?;
        self.trigger_module_event(module, AppEvent::APP_RUN)
    }

    pub fn command(&self) -> Ok<&str> {
        self.args
            .get("command")
            .unwrap()
            .ok_or("Argument 'command' not specified")?
            .into_ok()
    }

    pub fn register_command(
        &mut self,
        command: &'static str,
        module: AppModule<C>
    ) -> &mut Self {
        if self.commands.get(command).map(|m| fn_addr_eq(*m, module)) == Some(false) {
            panic!("App module command already registered: '{command}'");
        }
        self.commands.insert(command, module);
        self
    }

    pub fn unregister_command(&mut self, command: &str) -> Option<AppModule<C>> {
        self.commands.swap_remove(command)
    }

    pub fn register_module(&mut self, module: AppModule<C>) -> &mut Self {
        self.modules.insert(module);
        self
    }

    pub fn unregister_module(&mut self, module: &AppModule<C>) -> Option<AppModule<C>> {
        if let Some(command) = self
            .commands
            .iter()
            .find_map(|(c, m)| fn_addr_eq(*m, *module).then_some(c))
        {
            self.unregister_command(command);
        }

        if let Some(pos) = self.modules.get_index_of(module) {
            return self.modules.shift_remove_index(pos);
        }

        None
    }

    fn trigger_event(&mut self, event: AppEvent) -> Void {
        Env::is_debug().then(|| log::debug!("Raise event: {event:#?}"));

        for module in self.modules.clone() {
            module(self, event)?;
        }

        ok()
    }

    fn trigger_module_event(&mut self, module: AppModule<C>, event: AppEvent) -> Void {
        Env::is_debug()
            .then(|| log::debug!("Raise event: {event:#?} (module: {module:p})"));

        module(self, event)
    }

    pub fn get_module_by_command(&self, command: &str) -> Ok<AppModule<C>> {
        if let Some(module) = self.commands.get(command) {
            Ok(*module)
        } else if C::COMMAND == command
            && self.commands.is_empty()
            && let Some(module) = self.modules.first()
        {
            Ok(*module)
        } else {
            Err(format!("Invalid command: '{command}'"))?
        }
    }

    pub fn correct_command_name(&mut self) -> Void {
        let command = self.command()?;
        if self.commands.contains_key(&command) == false {
            let mut similar = self.commands.keys().filter(|c| c.starts_with(&command));
            if similar.clone().count() == 1 {
                let correct_command = similar.next().unwrap().to_string();
                Env::is_debug().then(|| {
                    log::trace!("Correct command name '{command}' to '{correct_command}'")
                });
                self.args.insert("command".into(), Some(correct_command));
            }
        }
        ok()
    }

    fn panic_handler(
        #[cfg(feature = "std")] info: &PanicHookInfo,
        #[cfg(not(feature = "std"))] info: &PanicInfo
    ) {
        eprintln!("PANIC: {info}");
        log::error!("{info}");
        unsafe {
            Di::from_static_mut().clear();
            Logger::from_static_mut().log_close();
        }
    }
}
