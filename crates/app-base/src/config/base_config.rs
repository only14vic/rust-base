use {
    super::LogConfig,
    crate::prelude::*,
    alloc::string::{String, ToString},
    serde::{Deserialize, Serialize}
};

#[derive(Debug, ExtendFromIter, Serialize, Deserialize)]
pub struct BaseConfig {
    pub language: String,
    pub timezone: String,
    #[serde(skip)]
    pub locales: IndexMap<String, Option<String>>,
    #[serde(skip)]
    pub log: LogConfig
}

impl Default for BaseConfig {
    fn default() -> Self {
        Self {
            language: "en".into(),
            timezone: "UTC".into(),
            locales: Default::default(),
            log: Default::default()
        }
    }
}

impl LoadEnv for BaseConfig {
    fn load_env(&mut self) -> Void {
        #[rustfmt::skip]
        self.extend(
            [
                ("language", getenv("LANG")),
                ("timezone", getenv("TZ"))
            ]
            .iter()
            .map(convert::tuple_option_str)
        );

        if self.language.len() > 2 {
            self.language = self.language[0..2].into();
        }
        self.language.make_ascii_lowercase();
        self.log.load_env()?;
        ok()
    }
}

impl LoadArgs for BaseConfig {
    fn load_args(&mut self, args: &Args) -> Void {
        #[rustfmt::skip]
        self.extend(
            [
                ("language", args.get("language")),
                ("timezone", args.get("timezone")),
            ]
            .iter().map(convert::tuple_option_option_str)
        );
        self.log.load_args(args)?;
        ok()
    }
}
