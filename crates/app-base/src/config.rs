use {
    crate::prelude::*,
    alloc::string::{String, ToString},
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

impl BaseConfig {
    pub fn load_env(&mut self) -> Void {
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

        ok()
    }
}

#[derive(Debug, Clone, SetFromIter)]
pub struct LogConfig {
    #[parse]
    pub level: LevelFilter,
    pub color: bool,
    pub file: Option<String>
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
            file: None
        }
    }
}

impl LogConfig {
    pub fn load_env(&mut self) -> Void {
        self.set_from_iter(
            [
                ("level", getenv("LOG_LEVEL")),
                ("file", getenv("LOG_FILE")),
                ("color", getenv("LOG_COLOR"))
            ]
            .iter()
            .map(|(k, v)| (*k, v.as_ref().map(String::as_str)))
        )?;

        ok()
    }
}
