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
        let migrator_config = &***config;
        let db_config = &***config;
        let tokio_config = (***config).as_ref() as &Arc<_>;
        let migrator = Migrator::<Postgres>::new(migrator_config, db_config);

        tokio_start(Some(tokio_config))?.block_on(async {
            match action {
                Self::APPLY => migrator.apply(count?).await?,
                Self::REVERT => migrator.revert(count?).await?,
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
            Self::APPLY => self.help_apply(app),
            Self::REVERT => self.help_revert(app),
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
    const APPLY: &str = "apply";
    const REVERT: &str = "revert";
    const STATUS: &str = "status";

    fn help_default(&self, app: &App<C>) {
        let config = app.config();
        let migrator_config: &Arc<MigratorConfig> = (***config).as_ref();

        println!(
            r#"
Usage: {bin} {cmd} [action] [options]

This command {desc}.

Actions:
    apply   - apply migrations
    revert  - revert migrations
    status  - show migrations status (default)

Options:
    -d, --dir       - path to migrations directory [{dir}]
    -n, --dry-run   - does not commit any SQL queries
    -s, --simple    - applies simple sort of SQL files
    -v, --verbose   - detailed output
    -q, --quiet     - quiet output
    -h, --help      - show usage help
"#,
            bin = config.dirs.exe_file(),
            cmd = Self::COMMAND,
            desc = Self::DESCRIPTION,
            dir = migrator_config.dir
        );
    }

    fn help_apply(&self, app: &App<C>) {
        let config = app.config();
        let migrator_config: &Arc<MigratorConfig> = (***config).as_ref();

        println!(
            r#"
Usage: {bin} {cmd} apply [count] [options]

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
            bin = config.dirs.exe_file(),
            cmd = Self::COMMAND,
            dir = migrator_config.dir
        );
    }

    fn help_revert(&self, app: &App<C>) {
        let config = app.config();
        let migrator_config: &Arc<MigratorConfig> = (***config).as_ref();

        println!(
            r#"
Usage: {bin} {cmd} revert [count] [options]

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
Usage: {bin} {cmd} status [options]

Displays migration statuses.

Options:
    -d, --dir       - path to migrations directory [{dir}]
    -s, --simple    - applies simple sort of SQL files
    -n, --dry-run   - does not commit any SQL queries
    -v, --verbose   - detailed output
    -q, --quiet     - quiet output
    -h, --help      - show usage help
"#,
            bin = config.dirs.exe_file(),
            cmd = Self::COMMAND,
            dir = migrator_config.dir
        );
    }
}
