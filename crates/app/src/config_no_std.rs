use {
    alloc::{fmt::Debug, string::String},
    app_base::prelude::*,
    serde::{Deserialize, Serialize}
};

#[derive(Debug, Default, ExtendFromIter, Serialize, Deserialize)]
pub struct Config {}

impl AppConfigExt for Config {}

impl Iter<'_, (&'static str, String)> for Config {
    fn iter(&'_ self) -> impl Iterator<Item = (&'static str, String)> {
        [].into_iter()
    }
}

impl InitArgs for Config {
    fn init_args(&mut self, _args: &mut Args) {}
}

impl LoadArgs for Config {
    fn load_args(&mut self, _args: &Args) {}
}

impl LoadDirs for Config {
    fn load_dirs(&mut self, _dirs: &Dirs) {}
}

impl LoadEnv for Config {
    fn load_env(&mut self) {}
}
