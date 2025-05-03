use {
    crate::prelude::*,
    alloc::{
        string::{String, ToString},
        vec::Vec
    },
    app_macros::SetFromIter,
    log::LevelFilter
};

#[derive(Debug, SetFromIter)]
pub struct BaseConfig {
    pub lang: String,
    pub timezone: String,
    pub log: LogConfig
}

impl Default for BaseConfig {
    fn default() -> Self {
        Self {
            lang: "en".into(),
            timezone: "UTC".into(),
            log: Default::default()
        }
    }
}

impl LoadEnv for BaseConfig {
    fn load_env(&mut self) -> Ok<&mut Self> {
        self.set_from_iter(
            [("lang", getenv("LANG")), ("timezone", getenv("TZ"))]
                .iter()
                .map(|(k, v)| (*k, v.as_ref().map(String::as_str)))
        )?;

        if self.lang.len() > 2 {
            self.lang = self.lang[0..2].into();
        }
        self.lang.make_ascii_lowercase();
        self.log.load_env()?;
        self.into_ok()
    }
}

#[derive(Debug, Clone, SetFromIter)]
pub struct LogConfig {
    #[parse]
    pub level: LevelFilter,
    pub color: bool,
    pub file: Option<String>,
    pub filter: Option<Vec<String>>
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: if cfg!(debug_assertions) {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            },
            color: false,
            file: None,
            filter: None
        }
    }
}

impl LoadEnv for LogConfig {
    fn load_env(&mut self) -> Ok<&mut Self> {
        self.set_from_iter(
            [
                ("level", getenv("LOG_LEVEL")),
                ("file", getenv("LOG_FILE")),
                ("color", getenv("LOG_COLOR")),
                ("filter", getenv("LOG_FILTER"))
            ]
            .iter()
            .map(|(k, v)| (*k, v.as_ref().map(String::as_str)))
        )?;
        self.into_ok()
    }
}

impl LoadArgs for LogConfig {
    fn load_args(&mut self, args: &Args) -> Ok<&mut Self> {
        #[rustfmt::skip]
        self.set_from_iter(
            [
                ("level", args.get("log-level")),
                ("file", args.get("log-file")),
            ]
            .iter().map(|(k, v)| {(
                *k, v.unwrap_or(&None).as_ref().map(String::as_str)
            )})
        )?;
        self.into_ok()
    }
}
