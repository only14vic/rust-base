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
pub struct AppSimpleConfig {
    pub custom: Option<Box<str>>
}

impl AppConfigExt for AppSimpleConfig {
    const COMMAND: &str = "run";
}

impl Iter<'_, (&'static str, String)> for AppSimpleConfig {
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

impl LoadArgs for AppSimpleConfig {
    fn init_args(&mut self, args: &mut Args) {
        #[rustfmt::skip]
        args.add_options([
            ("custom", None, None)
        ]).unwrap();
    }

    fn load_args(&mut self, args: &Args) {
        #[rustfmt::skip]
        self.extend([
                ("custom", args.get("custom"))
            ]
            .iter()
            .map(convert::tuple_result_option_str)
        );

        let list = [] as [&mut dyn LoadArgs; 0];

        for item in list {
            item.load_args(args);
        }
    }
}

impl LoadEnv for AppSimpleConfig {
    fn load_env(&mut self) {
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
            item.load_env();
        }
    }
}

impl LoadDirs for AppSimpleConfig {
    fn load_dirs(&mut self, dirs: &Dirs) {
        let list = [] as [&mut dyn LoadDirs; 0];

        for item in list {
            item.load_dirs(dirs);
        }
    }
}
