use {
    crate::{app::AppConfigExt, prelude::*},
    alloc::{
        boxed::Box,
        fmt::Display,
        string::{String, ToString},
        vec::Vec
    },
    app_macros::ExtendFromIter,
    core::fmt::Debug,
    serde::{Deserialize, Serialize}
};

#[derive(Debug, Default, ExtendFromIter, Serialize, Deserialize)]
pub struct AppConstomConfig {
    pub custom: Option<Box<str>>
}

impl AppConfigExt for AppConstomConfig {}

impl Iter<'_, (&'static str, String)> for AppConstomConfig {
    fn iter(&self) -> impl Iterator<Item = (&'static str, String)> {
        [(
            "custom",
            &self.custom.as_ref().map(|v| v.as_ref()).unwrap_or_default() as &dyn Display
        )]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
        .collect::<Vec<_>>()
        .into_iter()
    }
}

impl LoadArgs for AppConstomConfig {
    fn load_args(&mut self, args: &Args) -> Void {
        #[rustfmt::skip]
        self.extend(
            [
                ("custom", args.get("custom"))
            ]
            .iter()
            .map(convert::tuple_option_option_str)
        );

        let list = [] as [&mut dyn LoadArgs; 0];

        for item in list {
            item.load_args(args)?;
        }

        ok()
    }
}

impl LoadEnv for AppConstomConfig {
    fn load_env(&mut self) -> Void {
        #[rustfmt::skip]
        self.extend(
            [
                ("custom", getenv("CUSTOM")),
            ]
            .iter()
            .map(convert::tuple_option_str)
        );

        let list = [] as [&mut dyn LoadEnv; 0];

        for item in list {
            item.load_env()?;
        }

        ok()
    }
}

impl LoadDirs for AppConstomConfig {
    fn load_dirs(&mut self, dirs: &Dirs) -> Void {
        let list = [] as [&mut dyn LoadDirs; 0];

        for item in list {
            item.load_dirs(dirs)?;
        }

        ok()
    }
}
