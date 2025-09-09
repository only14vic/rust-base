use {
    crate::{app::AppConfigExt, prelude::*},
    alloc::{
        fmt::Display,
        string::{String, ToString},
        vec::Vec
    },
    app_macros::ExtendFromIter,
    core::{fmt::Debug, ptr::NonNull},
    serde::{Deserialize, Serialize}
};

#[derive(Debug, Default, ExtendFromIter, Serialize, Deserialize)]
pub struct AppConstomConfig {
    #[serde(skip)]
    pub custom: Option<NonNull<str>>
}

impl AppConfigExt for AppConstomConfig {}

impl<'a> From<&'a AppConstomConfig> for Vec<(&'static str, String)> {
    fn from(value: &'a AppConstomConfig) -> Self {
        [(
            "custom",
            &value
                .custom
                .as_ref()
                .map(|v| unsafe { v.as_ref() })
                .unwrap_or_default() as &dyn Display
        )]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
        .collect()
    }
}

impl<'a> From<&'a mut AppConstomConfig> for Vec<&'a mut dyn LoadDirs> {
    fn from(_value: &'a mut AppConstomConfig) -> Self {
        alloc::vec![]
    }
}

impl<'a> From<&'a mut AppConstomConfig> for Vec<&'a mut dyn LoadEnv> {
    fn from(_value: &'a mut AppConstomConfig) -> Self {
        alloc::vec![]
    }
}

impl<'a> From<&'a mut AppConstomConfig> for Vec<&'a mut dyn LoadArgs> {
    fn from(_value: &'a mut AppConstomConfig) -> Self {
        alloc::vec![]
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

        let list: Vec<&mut dyn LoadArgs> = self.into();

        for config in list {
            config.load_args(args)?;
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

        let list: Vec<&mut dyn LoadEnv> = self.into();

        for config in list {
            config.load_env()?;
        }

        ok()
    }
}

impl LoadDirs for AppConstomConfig {
    fn load_dirs(&mut self, dirs: &Dirs) -> Void {
        let list: Vec<&mut dyn LoadDirs> = self.into();

        for config in list {
            config.load_dirs(dirs)?;
        }

        ok()
    }
}
