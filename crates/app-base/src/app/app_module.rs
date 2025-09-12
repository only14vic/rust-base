use crate::prelude::*;

pub type AppModule<C> = fn(&mut App<C>, AppEvent) -> Void;

pub trait AppModuleExt: Default + Send + Sync + 'static {
    const COMMAND: &str;
    const DESCRIPTION: &str;

    type Config: AppConfigExt;

    fn handle(app: &mut App<Self::Config>, event: AppEvent) -> Void {
        if app.has::<Self>() == false {
            app.set(Self::default());
        }

        if event == AppEvent::APP_INIT && Self::COMMAND.is_empty() == false {
            app.register_command(Self::COMMAND, Self::handle);
        }

        let args = app.get_ref::<Args>().unwrap();
        let show_help = args.get("help").unwrap().is_some();
        let module = unsafe { &mut *(app.get_mut::<Self>().unwrap() as *mut Self) };

        match event {
            AppEvent::APP_INIT => module.init(app),
            AppEvent::APP_LOADED => module.loaded(app),
            AppEvent::APP_BOOT => module.boot(app),
            AppEvent::APP_RUN => {
                if show_help {
                    module.help(app)
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
    fn loaded(&mut self, app: &mut App<Self::Config>) -> Void {
        ok()
    }

    #[allow(unused_variables)]
    fn boot(&mut self, app: &mut App<Self::Config>) -> Void {
        ok()
    }

    fn run(&mut self, app: &mut App<Self::Config>) -> Void;

    #[allow(unused_variables)]
    fn end(&mut self, app: &mut App<Self::Config>) -> Void {
        ok()
    }

    #[allow(unused_variables)]
    fn help(&self, app: &mut App<Self::Config>) -> Void;
}
