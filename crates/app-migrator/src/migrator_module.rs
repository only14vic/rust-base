use {app_base::prelude::*, core::marker::PhantomData};

#[derive(Default)]
pub struct MigratorModule<C: AppConfigExt>(PhantomData<C>);

impl<C> AppModuleExt for MigratorModule<C>
where
    C: AppConfigExt
{
    const COMMAND: &str = "migrator";
    const DESCRIPTION: &str = "migrates SQL files";

    type Config = C;

    fn init(&mut self, app: &mut App<Self::Config>) -> Void {
        let args = app.get_mut::<Args>().unwrap();

        if Some(Self::COMMAND) == args.get("command").unwrap() {
            args.add_options([("action", &["2"][..], None)]).unwrap();
        }

        ok()
    }

    fn run(&mut self, _app: &mut App<Self::Config>) -> Void {
        dbg!("Migrator: OK");

        ok()
    }

    fn help(&self, app: &mut App<Self::Config>) -> Void {
        let config = app.config();

        println!(
            r#"
Usage: {bin} {cmd} [action] [options]

This command {desc}.

Actions:
    apply   - apply migrations (default)
    revert  - revert migrations
    status  - show migrations status

Options:
    -h, --help - show usage help
"#,
            bin = config.dirs.exe_file(),
            cmd = Self::COMMAND,
            desc = Self::DESCRIPTION
        );

        ok()
    }
}
