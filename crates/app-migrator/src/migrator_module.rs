use {app_base::prelude::*, core::marker::PhantomData};

#[derive(Default)]
pub struct MigratorModule<C: AppConfigExt>(PhantomData<C>);

impl<C> AppModuleExt for MigratorModule<C>
where
    C: AppConfigExt
{
    const COMMAND: &str = "migrator";

    type Config = C;

    fn init(&mut self, app: &mut App<Self::Config>) -> Void {
        let args = app.get_mut::<Args>().unwrap();

        if Some(Self::COMMAND) == args.get("command").unwrap() {
            args.add_options([("name", &["2"][..], None)]).unwrap();
        }

        ok()
    }

    fn run(&mut self, _app: &mut App<Self::Config>) -> Void {
        dbg!("Migrator: OK");

        ok()
    }

    fn help(&self, app: &mut App<Self::Config>) -> Void {
        let config = app.config();
        let bin = config.dirs.exe_file();
        let command = Self::COMMAND;

        println!(
            r#"
Usage: {bin} {command} [action] [options]

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
}
