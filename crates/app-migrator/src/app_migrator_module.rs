use app_base::prelude::*;

const COMMAND_NAME: &str = "migrator";

pub fn module_app_migrator<C>(app: &mut App<C>, event: AppEvent) -> Void
where
    C: AppConfigExt
{
    match event {
        AppEvent::APP_INIT => {
            app.register_command(COMMAND_NAME, module_app_migrator);

            let args = app.get_mut::<Args>().unwrap();
            if Some(COMMAND_NAME) == args.get("command").unwrap() {
                args.add_options([("name", &["2"][..], None)]).unwrap();
            }
        },
        AppEvent::APP_RUN => {
            let args = app.get_ref::<Args>().unwrap();
            if args.get("help").unwrap().is_some() {
                show_help(app)?;
            } else {
                dbg!("OK");
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
Usage: {exe_file} {COMMAND_NAME} [action] [options]

Actions:
    apply       - apply migrations (default)
    revert      - revert migrations
    status      - show migrations status

Options:
    -h, --help - show usage help
"#,
    );
    ok()
}
