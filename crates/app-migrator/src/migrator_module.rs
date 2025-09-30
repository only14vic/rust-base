use {
    crate::{Migrator, MigratorConfig, MigratorConfigExt},
    app_async::tokio_start,
    app_base::prelude::*,
    core::marker::PhantomData,
    sqlx::Postgres,
    std::sync::Arc
};

#[derive(Default)]
pub struct MigratorModule<C: MigratorConfigExt>(PhantomData<C>);

impl<C> AppModuleExt for MigratorModule<C>
where
    C: MigratorConfigExt
{
    const COMMAND: &str = MigratorConfig::COMMAND;
    const DESCRIPTION: &str = "migrates SQL files to database";

    type Config = C;

    fn init(&mut self, app: &mut App<Self::Config>) -> Void {
        if Self::COMMAND == app.command()? {
            #[rustfmt::skip]
            app.args_mut().add_options([
                ("action", "2".into(), None),
                ("count", "3".into(), None)
            ]).unwrap();
        }

        ok()
    }

    fn setup(&mut self, app: &mut App<Self::Config>) -> Void {
        let config = unsafe { app.config_static() };
        let migrator_config = app
            .config_mut()
            .unwrap()
            .get_mut::<MigratorConfig>()
            .unwrap();
        migrator_config.load_config(config);
        ok()
    }

    fn run(&mut self, app: &mut App<Self::Config>) -> Void {
        let args = app.args();
        let action = args.get("action").unwrap().unwrap_or(Self::STATUS);
        let count = args
            .get("count")
            .unwrap()
            .map(|v| {
                v.parse()
                    .map_err(|e| format!("Invalid value of count: '{v}' - {e}"))
            })
            .transpose();

        let config = app.config();
        let tokio_config = (***config).as_ref() as &Arc<_>;
        let migrator = Migrator::<Postgres>::new(config);

        tokio_start(Some(tokio_config))?.block_on(async {
            match action {
                Self::UP => migrator.up(count?).await?,
                Self::DOWN => migrator.down(count?).await?,
                Self::STATUS => migrator.status().await?,
                _ => Err(format!("Invalid action: '{action}'"))?
            }

            ok()
        })
    }

    fn help(&self, app: &mut App<Self::Config>) -> Void {
        let args = app.args();
        let action = args.get("action").unwrap().unwrap_or_default();

        match action {
            Self::UP => self.help_up(app),
            Self::DOWN => self.help_down(app),
            Self::STATUS => self.help_status(app),
            _ => self.help_default(app)
        }

        ok()
    }
}

impl<C> MigratorModule<C>
where
    C: MigratorConfigExt
{
    const UP: &str = "up";
    const DOWN: &str = "down";
    const STATUS: &str = "status";

    fn help_default(&self, app: &App<C>) {
        let config = app.config();
        let migrator_config: &Arc<MigratorConfig> = (***config).as_ref();

        println!(
            r#"
Usage: {bin} {cmd} [action] [options]

This command {desc}.

Actions:
    {:<len$} - apply migrations
    {:<len$} - revert migrations
    {:<len$} - show migrations status (default)

Options:
    -d, --dir       - path to migrations directory [{dir}]
    -n, --dry-run   - does not commit any SQL queries
    -s, --simple    - applies simple sort of SQL files
    -v, --verbose   - detailed output
    -q, --quiet     - quiet output
    -h, --help      - show usage help
"#,
            Self::UP,
            Self::DOWN,
            Self::STATUS,
            len = Self::STATUS.len(),
            bin = config.dirs.exe_file(),
            cmd = Self::COMMAND,
            desc = Self::DESCRIPTION,
            dir = migrator_config.dir
        );
    }

    fn help_up(&self, app: &App<C>) {
        let config = app.config();
        let migrator_config: &Arc<MigratorConfig> = (***config).as_ref();

        println!(
            r#"
Usage: {bin} {cmd} {} [count] [options]

Applies migrations.

Arguments:
    count      - number of migrations to be applied

Options:
    -d, --dir       - path to migrations directory [{dir}]
    -n, --dry-run   - does not commit any SQL queries
    -s, --simple    - applies simple sort of SQL files
    -v, --verbose   - detailed output
    -q, --quiet     - quiet output
    -h, --help      - show usage help
"#,
            Self::UP,
            bin = config.dirs.exe_file(),
            cmd = Self::COMMAND,
            dir = migrator_config.dir
        );
    }

    fn help_down(&self, app: &App<C>) {
        let config = app.config();
        let migrator_config: &Arc<MigratorConfig> = (***config).as_ref();

        println!(
            r#"
Usage: {bin} {cmd} {} [count] [options]

Reverts migrations.

Arguments:
    count      - number of migrations to be reverted

Options:
    -d, --dir       - path to migrations directory [{dir}]
    -n, --dry-run   - does not commit any SQL queries
    -v, --verbose   - detailed output
    -q, --quiet     - quiet output
    -h, --help      - show usage help
"#,
            Self::DOWN,
            bin = config.dirs.exe_file(),
            cmd = Self::COMMAND,
            dir = migrator_config.dir
        );
    }

    fn help_status(&self, app: &App<C>) {
        let config = app.config();
        let migrator_config: &Arc<MigratorConfig> = (***config).as_ref();

        println!(
            r#"
Usage: {bin} {cmd} {} [options]

Displays migration statuses.

Options:
    -d, --dir       - path to migrations directory [{dir}]
    -s, --simple    - applies simple sort of SQL files
    -n, --dry-run   - does not commit any SQL queries
    -v, --verbose   - detailed output
    -q, --quiet     - quiet output
    -h, --help      - show usage help
"#,
            Self::STATUS,
            bin = config.dirs.exe_file(),
            cmd = Self::COMMAND,
            dir = migrator_config.dir
        );
    }
}
