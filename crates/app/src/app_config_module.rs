use {crate::*, alloc::format, app_base::prelude::*};

pub const MODULE_APP_CONFIG: AppModule = module_app_config;

fn module_app_config(app: &mut App, event: AppEvent) -> Void {
    match event {
        AppEvent::APP_INIT => {
            app.register_command("config", MODULE_APP_CONFIG);
            ok()
        },
        AppEvent::APP_RUN => {
            let config = app.config();
            let args = app.get_ref::<Args>().unwrap();
            let name = args.get("value").ok_or("Undefined argument 'value'")?;

            if args.get("help").unwrap().is_some() {
                let exe_file = config.dirs.exe_file();
                println!(
                    r#"
Usage: {exe_file} config [name] [options]

This command display config options.
If "name" is defined then it displays the value of the option by its name.

Options:
    -h, --help - show usage help
"#,
                );
            } else if let Some(name) = name {
                if let Some((.., value)) = config.iter().find(|(k, _)| *k == name.as_str()) {
                    println!("{value}");
                } else {
                    Err(format!("Invalid config option name: {name}"))?;
                }
            } else {
                for (k, v) in config.iter() {
                    println!("{k}={v}");
                }
            }
            ok()
        },
        _ => ok()
    }
}
