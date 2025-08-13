use {crate::*, alloc::format, app_base::prelude::*};

pub const MODULE_APP_CONFIG: AppModule = module_app_config;

fn module_app_config(app: &mut App, event: AppEvent) -> Void {
    match event {
        AppEvent::APP_INIT => {
            app.register_command("config", MODULE_APP_CONFIG);
        },
        AppEvent::APP_RUN => {
            let args = app.get_ref::<Args>().unwrap();
            let name = args.get("value").ok_or("Undefined argument 'value'")?;

            if args.get("help").unwrap().is_some() {
                show_help(app)?;
            } else if let Some(name) = name {
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

This command display config options.

Arguments:
    name - if defined then it displays the value of the option by its name

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
