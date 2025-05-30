use {
    crate::prelude::*,
    alloc::{
        string::{String, ToString},
        vec::Vec
    },
    log::LevelFilter
};

#[derive(Debug, Extend)]
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
    fn load_env(&mut self) -> Ok<()> {
        self.extend(
            [("lang", getenv("LANG")), ("timezone", getenv("TZ"))]
                .iter()
                .map(convert::tuple_option_string_to_str)
        );

        if self.lang.len() > 2 {
            self.lang = self.lang[0..2].into();
        }
        self.lang.make_ascii_lowercase();
        self.log.load_env()?;
        ok()
    }
}

#[derive(Debug, Clone, ExtendFromIter)]
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
    fn load_env(&mut self) -> Ok<()> {
        self.extend(
            [
                ("level", getenv("LOG_LEVEL")),
                ("file", getenv("LOG_FILE")),
                ("color", getenv("LOG_COLOR")),
                ("filter", getenv("LOG_FILTER"))
            ]
            .iter()
            .map(convert::tuple_option_string_to_str)
        );
        ok()
    }
}

impl LoadArgs for LogConfig {
    fn load_args(&mut self, args: &Args) -> Ok<()> {
        #[rustfmt::skip]
        self.extend(
            [
                ("level", args.get("log-level")),
                ("file", args.get("log-file")),
            ]
            .iter().map(convert::tuple_option_option_string_to_str)
        );
        ok()
    }
}
