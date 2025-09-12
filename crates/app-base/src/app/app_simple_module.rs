use {crate::prelude::*, core::marker::PhantomData};

#[derive(Default)]
pub struct AppSimpleModule<C>(PhantomData<C>);

impl<C> AppModuleExt for AppSimpleModule<C>
where
    C: AppConfigExt
{
    const COMMAND: &str = C::DEFAULT_COMMAND;

    type Config = C;

    fn run(&mut self, _app: &mut App<Self::Config>) -> Void {
        println!("Simple module says: Hello, World!");
        ok()
    }

    fn help(&self, app: &mut App<Self::Config>) -> Void {
        let config = app.config();
        let bin = config.dirs.exe_file();
        let command = Self::COMMAND;

        println!(
            r#"
Usage: {bin} {command} [options]

This is simple command.

Options:
    -h, --help  - show usage help
"#,
        );

        ok()
    }
}
