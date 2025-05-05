use {
    crate::{App, *},
    app_base::prelude::*,
    app_web::HttpServer
};

#[derive(Default)]
pub struct MainModule;

impl AppModuleExt for MainModule {
    const COMMAND: &str = "run";
    const DESCRIPTION: &str = "run http server";

    type Config = Config;

    fn setup(&mut self, app: &mut App) -> Void {
        app.unregister_command(WebModule::<Self::Config>::COMMAND);
        ok()
    }

    fn run(&mut self, app: &mut App) -> Void {
        let config = app.config();

        Dirs::mkdir(&config.dirs.var)?;
        Dirs::mkdir(&config.dirs.run)?;
        Dirs::mkdir(&config.dirs.log)?;
        Dirs::mkdir(&config.dirs.tmp)?;
        Dirs::mkdir(&config.dirs.cache)?;
        Dirs::mkdir(&config.dirs.state)?;
        Dirs::mkdir(&config.dirs.user_config)?;

        let mut server = app.take::<HttpServer<Self::Config>>().unwrap();
        server.with_defaults(app);
        server.run_with_runtime()
    }

    fn help(&self, app: &mut App) -> Void {
        let config = app.config();

        println!(
            r#"
Usage: {bin} [command] [options]

Version: {name} {version}

Commands:
    {:<len$} - {} (default)
    {:<len$} - {}
    {:<len$} - {}

Options:
    -h, --help      - show usage help
    --env-file file - loads env vars from file
    --debug         - enable debuging
    --version       - show current version
"#,
            Self::COMMAND,
            Self::DESCRIPTION,
            AppConfigModule::<Self::Config>::COMMAND,
            AppConfigModule::<Self::Config>::DESCRIPTION,
            MigratorModule::<Self::Config>::COMMAND,
            MigratorModule::<Self::Config>::DESCRIPTION,
            len = 10,
            bin = config.dirs.exe_file(),
            version = config.version,
            name = config.name,
        );

        ok()
    }
}
