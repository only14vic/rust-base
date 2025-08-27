use {crate::*, app_base::prelude::*, core::ffi::c_uint};

pub const MODULE_APP_CONFIG: AppModule = module_app_config;

#[unsafe(no_mangle)]
extern "C" fn module_app_config_c(app: *mut App, event: AppEvent) -> c_uint {
    module_app_config(unsafe { &mut *app }, event)
        .map(|_| 0)
        .unwrap_or_else(|e| panic!("{e}"))
}

fn module_app_config(app: &mut App, event: AppEvent) -> Void {
    match event {
        AppEvent::APP_INIT => {
            app.register_command("config", MODULE_APP_CONFIG);
            let args = app.get_mut::<Args>()?.unwrap();
            args.add_options([
                ("log-level", &[][..], None),
                ("log-color", &[], None),
                ("log-file", &[], None),
                ("log-filter", &[], None),
                ("language", &[], None),
                ("timezone", &[], None),
                ("home-dir", &[], None),
                ("config-dir", &[], None),
                ("user-config-dir", &[], None),
                ("log-dir", &[], None),
                ("var-dir", &[], None),
                ("run-dir", &[], None),
                ("data-dir", &[], None),
                ("cache-dir", &[], None),
                ("state-dir", &[], None),
                ("tmp-dir", &[], None),
                ("tokio-threads", &[], None),
                ("actix-socket", &[], None),
                ("actix-listen", &[], None),
                ("actix-port", &[], None),
                ("actix-threads", &[], None),
                ("db-url", &[], None),
                ("web-host", &[], None),
                ("web-hostname", &[], None),
                ("web-base-url", &[], None),
                ("web-static-dir", &[], None),
                ("web-static-path", &[], None)
            ])
            .unwrap();
            if Some("config") == args.get_option("command").unwrap() {
                args.add_options([("name", &["2"][..], None)]).unwrap();
            }
        },
        AppEvent::APP_RUN => {
            let args = app.get_ref::<Args>().unwrap();
            let name = args.get_option("name").unwrap();

            if args.get_option("help").unwrap().is_some() {
                show_help(app)?;
            } else {
                show_config(app, name)?;
            }
        },
        _ => ()
    }
    ok()
}

fn show_help(app: &App) -> Void {
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

fn show_config(app: &App, name: Option<&str>) -> Void {
    let config = app.config();
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
