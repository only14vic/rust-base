use {
    crate::prelude::*,
    alloc::{format, vec::Vec},
    core::{ffi::c_void, marker::PhantomData, ptr::null}
};

#[unsafe(no_mangle)]
extern "C" fn module_app_config(app: *mut app_c::App, event: AppEvent) -> *const c_void {
    match AppConfigModule::handle(unsafe { &mut *app }, event) {
        Ok(..) => null(),
        Err(e) => panic!("{e}")
    }
}

#[derive(Default)]
pub struct AppConfigModule<C: AppConfigExt>(PhantomData<C>);

impl<C> AppModuleExt for AppConfigModule<C>
where
    C: AppConfigExt
{
    const COMMAND: &str = AppConfig::<C>::COMMAND;
    const DESCRIPTION: &str = "displays config options";

    type Config = C;

    fn init(&mut self, app: &mut App<Self::Config>) -> Void {
        if Self::COMMAND == app.command()? {
            app.args_mut()
                .add_options([("name", "2".into(), None)])
                .unwrap();
        }

        ok()
    }

    fn run(&mut self, app: &mut App<Self::Config>) -> Void {
        let config = app.config().as_ref();
        let args = app.args();
        let name = args.get("name").unwrap();
        let iter = config.iter();

        let mut list: Vec<_> = match name {
            Some(name) => iter.filter(|(k, _)| k.contains(name)).collect(),
            None => iter.collect()
        };
        let count = list.len();

        if let Some(name) = name
            && list.is_empty()
        {
            Err(format!("Invalid config option name: {name}"))?;
        }

        list.sort_by_key(|(k, _)| *k);

        for (k, v) in list {
            if k.is_empty() {
                println!("{v}");
            } else if v.contains("\n") {
                v.split_terminator('\n').for_each(|v| {
                    println!("{k}.{v}");
                });
            } else if count == 1 && name == Some(k) {
                println!("{v}");
            } else {
                println!("{k}={v}");
            }
        }

        ok()
    }

    fn help(&self, app: &mut App<Self::Config>) -> Void {
        let config = app.config();

        println!(
            r#"
Usage: {bin} {cmd} [name] [options]

This command {desc}.

Arguments:
    name - if defined, then it displays option(s) filtered by name

Options:
    -h, --help  - show usage help
"#,
            bin = config.dirs.exe_file(),
            cmd = Self::COMMAND,
            desc = Self::DESCRIPTION
        );

        ok()
    }
}
