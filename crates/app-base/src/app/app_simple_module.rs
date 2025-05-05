use {crate::prelude::*, core::marker::PhantomData};

#[derive(Default)]
pub struct AppSimpleModule<C: AppConfigExt>(PhantomData<C>);

impl<C> AppModuleExt for AppSimpleModule<C>
where
    C: AppConfigExt
{
    const COMMAND: &str = "run";
    const DESCRIPTION: &str = "simple command";

    type Config = C;

    fn run(&mut self, _app: &mut App<Self::Config>) -> Void {
        println!("Simple module says: Hello, World!");
        ok()
    }
}
