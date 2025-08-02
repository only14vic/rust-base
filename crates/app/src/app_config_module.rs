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
                let exe = args
                    .get("exe")
                    .ok_or("Undefined argument 'exe'")?
                    .as_ref()
                    .map(|s| {
                        let mut s = s.clone();
                        s.replace_range(0..=s.rfind("/").unwrap_or(0), "");
                        s
                    })
                    .ok_or("Argument 'exe' must be defined")?;
                println!(
                    r#"
Usage: {exe} config [name] [options]

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
