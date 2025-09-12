use {crate::prelude::*, core::marker::PhantomData};

#[derive(Default)]
pub struct AppSimpleModule<C: AppConfigExt>(PhantomData<C>);

impl<C> AppModuleExt for AppSimpleModule<C>
where
    C: AppConfigExt
{
    const COMMAND: &str = C::DEFAULT_COMMAND;
    const DESCRIPTION: &str = "simple example command";

    type Config = C;

    fn run(&mut self, _app: &mut App<Self::Config>) -> Void {
        println!("Simple module says: Hello, World!");
        ok()
    }

    fn help(&self, app: &mut App<Self::Config>) -> Void {
        let config = app.config();

        println!(
            r#"
Usage: {bin} [command] [options]

Version: {name} ({version})

Commands:
    {:<len$} - {} (default)

Options:
    -h, --help  - show usage help
"#,
            Self::COMMAND,
            Self::DESCRIPTION,
            len = 5,
            bin = config.dirs.exe_file(),
            version = config.base.version,
            name = config.base.name,
        );

        ok()
    }
}
