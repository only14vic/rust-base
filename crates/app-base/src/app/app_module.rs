use {
    crate::{prelude::*, type_name_simple},
    alloc::format,
    core::any::type_name
};

pub type AppModule<C> = fn(&mut App<C>, AppEvent) -> Void;

pub trait AppModuleExt: Default + Send + Sync + 'static {
    const COMMAND: &str = "";
    const DESCRIPTION: &str = "";

    type Config: AppConfigExt;

    fn handle(app: &mut App<Self::Config>, event: AppEvent) -> Void {
        if event == AppEvent::APP_INIT {
            if app.has::<Self>() == false {
                app.add(Self::default());
            }

            if Self::COMMAND.is_empty() == false {
                app.register_command(Self::COMMAND, Self::handle);
            }
        }

        if app.has::<Self>() == false {
            Err(format!(
                "Module object does not exist in app: '{}'",
                type_name_simple!(Self)
            ))?;
        }

        let args = app.args();
        let show_help = args.get_flag("help").unwrap();
        let show_version = args.get_flag("version").unwrap();
        let module = unsafe { &mut *(app.get_mut::<Self>().unwrap() as *mut Self) };

        Env::is_debug().then(|| {
            log::trace!(
                "{}::{}",
                type_name_simple!(Self),
                match event {
                    AppEvent::APP_INIT => "init()",
                    AppEvent::APP_BOOT => "boot()",
                    AppEvent::APP_SETUP => "setup()",
                    AppEvent::APP_RUN =>
                        if show_help {
                            "help()"
                        } else if show_version {
                            "version()"
                        } else {
                            "run()"
                        },
                    AppEvent::APP_END => "end()"
                }
            )
        });

        match event {
            AppEvent::APP_INIT => module.init(app),
            AppEvent::APP_BOOT => module.boot(app),
            AppEvent::APP_SETUP => module.setup(app),
            AppEvent::APP_RUN => {
                if show_help {
                    module.help(app)
                } else if show_version {
                    module.version(app)
                } else {
                    module.run(app)
                }
            },
            AppEvent::APP_END => module.end(app)
        }
    }

    #[allow(unused_variables)]
    fn init(&mut self, app: &mut App<Self::Config>) -> Void {
        ok()
    }

    #[allow(unused_variables)]
    fn setup(&mut self, app: &mut App<Self::Config>) -> Void {
        ok()
    }

    #[allow(unused_variables)]
    fn boot(&mut self, app: &mut App<Self::Config>) -> Void {
        ok()
    }

    #[allow(unused_variables)]
    fn run(&mut self, app: &mut App<Self::Config>) -> Void {
        ok()
    }

    #[allow(unused_variables)]
    fn end(&mut self, app: &mut App<Self::Config>) -> Void {
        ok()
    }

    fn version(&self, app: &mut App<Self::Config>) -> Void {
        println!(
            "{name} {version}",
            name = app.config().name,
            version = app.config().version
        );
        ok()
    }

    fn help(&self, app: &mut App<Self::Config>) -> Void {
        let config = app.config();

        println!(
            r#"
Usage: {bin} [command] [options]

Version: {name} {version}

This command {desc}

Commands:
    {:<len$} - {desc}

Options:
    -h, --help      - show usage help
    --env-file file - loads env vars from file
    --debug         - enable debuging
    --version       - show current version
"#,
            Self::COMMAND,
            desc = Self::DESCRIPTION,
            len = 5,
            bin = config.dirs.exe_file(),
            name = config.name,
            version = config.version,
        );

        ok()
    }
}
