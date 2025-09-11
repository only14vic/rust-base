use {
    crate::prelude::*,
    alloc::{format, vec::Vec},
    core::ffi::c_uint
};

#[unsafe(no_mangle)]
extern "C" fn module_app_config_c(app: *mut app_c::App, event: AppEvent) -> c_uint {
    match module_app_config(unsafe { &mut *app }, event) {
        Ok(..) => 0,
        Err(e) => panic!("{e}")
    }
}

pub fn module_app_config<C>(app: &mut App<C>, event: AppEvent) -> Void
where
    C: AppConfigExt
{
    match event {
        AppEvent::APP_INIT => {
            app.register_command("config", module_app_config);

            let args = app.get_mut::<Args>()?.unwrap();
            if Some("config") == args.get("command").unwrap() {
                args.add_options([("name", &["2"][..], None)]).unwrap();
            }
        },
        AppEvent::APP_RUN => {
            let args = app.get_ref::<Args>().unwrap();
            let name = args.get("name").unwrap();

            if args.get("help").unwrap().is_some() {
                show_help(app)?;
            } else {
                show_config(app, name)?;
            }
        },
        _ => ()
    }
    ok()
}

fn show_help<C>(app: &App<C>) -> Void
where
    C: AppConfigExt
{
    let config = app.config();
    let exe_file = config.dirs.exe_file();
    println!(
        r#"
Usage: {exe_file} config [name] [options]

This command displays config options.

Arguments:
    name - if defined, then it displays option(s) filtered by name

Options:
    -h, --help  - show usage help
"#,
    );
    ok()
}

fn show_config<C>(app: &App<C>, name: Option<&str>) -> Void
where
    C: AppConfigExt
{
    let config = app.config().as_ref();
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
