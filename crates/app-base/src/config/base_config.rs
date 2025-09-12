use {
    super::LogConfig,
    crate::prelude::*,
    alloc::{
        boxed::Box,
        format,
        string::{String, ToString},
        vec::Vec
    },
    core::fmt::Display,
    serde::{Deserialize, Serialize}
};

#[derive(Debug, ExtendFromIter, Serialize, Deserialize)]
pub struct BaseConfig {
    pub name: Box<str>,
    pub version: Box<str>,
    pub language: String,
    pub timezone: String,
    pub locales: IndexMap<String, Option<String>>,
    #[serde(skip)]
    pub log: LogConfig
}

impl AppConfigExt for BaseConfig {}

impl Default for BaseConfig {
    fn default() -> Self {
        Self {
            name: env!("APP_NAME").trim_matches(['\'', '"']).into(),
            version: env!("CARGO_PKG_VERSION").into(),
            language: "en".into(),
            timezone: "UTC".into(),
            locales: Default::default(),
            log: Default::default()
        }
    }
}

impl Iter<'_, (&'static str, String)> for BaseConfig {
    fn iter(&self) -> impl Iterator<Item = (&'static str, String)> {
        let env = Env::from_static();

        [
            // env
            ("env.env", &env.env as &dyn Display),
            ("env.is_prod", &env.is_prod),
            ("env.is_dev", &env.is_dev),
            ("env.is_debug", &env.is_debug),
            ("env.is_release", &env.is_release),
            //
            // base
            ("base.name", &self.name),
            ("base.version", &self.version),
            ("base.language", &self.language),
            (
                "base.locales",
                Box::leak(Box::new(
                    self.locales
                        .iter()
                        .map(|(n, m)| format!("{n}={}", m.as_ref().unwrap_or(&"".into())))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            ),
            ("base.timezone", &self.timezone),
            //
            // log
            ("base.log.level", &self.log.level),
            ("base.log.color", &self.log.color),
            (
                "base.log.filter",
                Box::leak(Box::new(
                    self.log
                        .filter
                        .as_ref()
                        .map(|v| v.join(","))
                        .unwrap_or_default()
                ))
            ),
            (
                "base.log.file",
                Box::leak(Box::new(self.log.file.as_deref().unwrap_or_default()))
            )
        ]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
    }
}

impl LoadArgs for BaseConfig {
    fn init_args(&mut self, args: &mut Args) {
        args.add_options([
            ("language", &[][..], None),
            ("timezone", &[], None),
            ("locales", &[], None)
        ])
        .unwrap();

        self.log.init_args(args);
    }

    fn load_args(&mut self, args: &Args) {
        #[rustfmt::skip]
        self.extend(
            [
                ("language", args.get("language")),
                ("timezone", args.get("timezone")),
                ("locales", args.get("locales")),
            ]
            .iter().map(convert::tuple_result_option_str)
        );

        self.log.load_args(args);
    }
}

impl LoadEnv for BaseConfig {
    fn load_env(&mut self) {
        #[rustfmt::skip]
        self.extend(
            [
                ("language", getenv("LANG")),
                ("timezone", getenv("TZ")),
                ("locales", getenv("LOCALES"))
            ]
            .iter()
            .map(convert::tuple_option_str)
        );

        if self.language.len() > 2 {
            self.language = self.language[0..2].into();
        }
        self.language.make_ascii_lowercase();
        self.log.load_env();
    }
}

impl LoadDirs for BaseConfig {
    fn load_dirs<'a>(&'a mut self, dirs: &'a Dirs) {
        self.log.load_dirs(dirs);
    }
}
