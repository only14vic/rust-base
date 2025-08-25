use {crate::*, alloc::format, app_base::prelude::*, core::ffi::c_uint};

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
            ])?;
            if Some("config") == args.get_option("command") {
                args.add_options([("name", &["2"][..], None)])?;
            }
        },
        AppEvent::APP_RUN => {
            let args = app.get_ref::<Args>().unwrap();

            if args.get_option("help").is_some() {
                show_help(app)?;
            } else if let Some(name) = args.get_option("name") {
                show_config_option(app, name)?;
            } else {
                show_config(app)?;
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
    name - if defined, then it displays the value of the option by its name

Options:
    -h, --help  - show usage help
"#,
    );
    ok()
}

fn show_config(app: &App) -> Void {
    let config = app.config();
    for (k, v) in config.iter() {
        if k.is_empty() {
            println!("{v}");
        } else if v.contains("\n") {
            v.split_terminator('\n').for_each(|v| {
                println!("{k}.{v}");
            });
        } else {
            println!("{k}={v}");
        }
    }
    ok()
}

fn show_config_option(app: &App, name: &str) -> Void {
    let config = app.config();
    if let Some((.., value)) = config.iter().find(|(k, _)| *k == name) {
        println!("{value}");
    } else {
        Err(format!("Invalid config option name: {name}"))?;
    }
    ok()
}
