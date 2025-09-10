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

impl<'a> IterMut<'a, &'a mut dyn LoadDirs> for AppConstomConfig {
    fn iter_mut(&'a mut self) -> impl Iterator<Item = &'a mut dyn LoadDirs> {
        [].into_iter()
    }
}

impl<'a> IterMut<'a, &'a mut dyn LoadEnv> for AppConstomConfig {
    fn iter_mut(&'a mut self) -> impl Iterator<Item = &'a mut dyn LoadEnv> {
        [].into_iter()
    }
}

impl<'a> IterMut<'a, &'a mut dyn LoadArgs> for AppConstomConfig {
    fn iter_mut(&'a mut self) -> impl Iterator<Item = &'a mut dyn LoadArgs> {
        [].into_iter()
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

        let list = <Self as IterMut<&mut dyn LoadArgs>>::iter_mut(self);

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

        let list = <Self as IterMut<&mut dyn LoadEnv>>::iter_mut(self);

        for item in list {
            item.load_env()?;
        }

        ok()
    }
}

impl LoadDirs for AppConstomConfig {
    fn load_dirs(&mut self, dirs: &Dirs) -> Void {
        let list = <Self as IterMut<&mut dyn LoadDirs>>::iter_mut(self);

        for item in list {
            item.load_dirs(dirs)?;
        }

        ok()
    }
}
