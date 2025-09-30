use {
    crate::{App, *},
    app_base::prelude::*
};

#[derive(Default)]
pub struct MainModule;

impl AppModuleExt for MainModule {
    const COMMAND: &str = "help";
    const DESCRIPTION: &str = "show help";

    type Config = Config;

    fn boot(&mut self, app: &mut App) -> Void {
        let mut mkdirs = [Self::COMMAND, AppConfigModule::<Self::Config>::COMMAND]
            .contains(&app.command()?)
            == false;

        mkdirs &= app.args().get_flag("help").unwrap() != true;
        mkdirs &= app.args().get_flag("version").unwrap() != true;

        if mkdirs {
            let config = app.config();

            Dirs::mkdir(&config.dirs.var)?;
            Dirs::mkdir(&config.dirs.run)?;
            Dirs::mkdir(&config.dirs.log)?;
            Dirs::mkdir(&config.dirs.tmp)?;
            Dirs::mkdir(&config.dirs.cache)?;
            Dirs::mkdir(&config.dirs.state)?;
            Dirs::mkdir(&config.dirs.user_config)?;
        }

        ok()
    }

    #[allow(unused_variables)]
    fn setup(&mut self, app: &mut App) -> Void {
        #[cfg(feature = "web")]
        {
            let init_runtime = self.init_runtime(app);
            if let Ok(web_module) = app.get_mut::<WebModule<Self::Config>>() {
                web_module.enable_defaults = true;
                web_module.with_init_runtime(init_runtime);
            }
        }

        ok()
    }

    fn run(&mut self, app: &mut App) -> Void {
        self.help(app)
    }

    fn help(&self, app: &mut App) -> Void {
        let config = app.config();
        const LEN: usize = 10;

        println!(
            r#"
Usage: {bin} [command] [options]

Version: {name} {version}

Commands:
    {:<LEN$} - {} (default)
    {:<LEN$} - {}"#,
            Self::COMMAND,
            Self::DESCRIPTION,
            AppConfigModule::<Self::Config>::COMMAND,
            AppConfigModule::<Self::Config>::DESCRIPTION,
            bin = config.dirs.exe_file(),
            version = config.version,
            name = config.name,
        );

        #[cfg(feature = "web")]
        println!(
            "    {:<LEN$} - {}",
            WebModule::<Self::Config>::COMMAND,
            WebModule::<Self::Config>::DESCRIPTION,
        );

        #[cfg(feature = "migrator")]
        println!(
            "    {:<LEN$} - {}",
            MigratorModule::<Self::Config>::COMMAND,
            MigratorModule::<Self::Config>::DESCRIPTION,
        );

        #[cfg(feature = "desktop")]
        println!(
            "    {:<LEN$} - {}",
            DesktopModule::<Self::Config>::COMMAND,
            DesktopModule::<Self::Config>::DESCRIPTION,
        );

        println!(
            r#"
Options:
    -h, --help      - show usage help
    --env-file file - loads env vars from file
    --debug         - enable debuging
    --version       - show current version
"#
        );

        ok()
    }
}

#[cfg(feature = "web")]
use {
    app_async::{
        db::{DbConfig, DbNotifyListener, db_pool},
        queue::QueueListener
    },
    futures::future::LocalBoxFuture,
    std::sync::Arc
};

impl MainModule {
    #[cfg(feature = "web")]
    fn init_runtime(
        &self,
        app: &mut App
    ) -> impl Fn() -> LocalBoxFuture<'static, Void> + Send + Sync + 'static {
        const NOTIFY_CHANNELS: [&str; 1] = ["app"];
        let db_config = app.config().get::<DbConfig>().clone();

        move || {
            let db_config = db_config.clone();

            async move {
                DbNotifyListener::new(
                    NOTIFY_CHANNELS,
                    &db_pool(Some(&db_config)).await?,
                    Arc::new(QueueListener::handle)
                )
                .start()
                .await;
                ok()
            }
            .into_pin_box()
        }
    }
}
