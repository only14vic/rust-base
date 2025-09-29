use {
    crate::prelude::*,
    alloc::{
        string::{String, ToString},
        vec::Vec
    },
    log::LevelFilter
};

#[derive(Debug, Clone, ExtendFromIter)]
pub struct LogConfig {
    #[extend_parse]
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

impl LoadArgs for LogConfig {
    fn init_args(&mut self, args: &mut Args) {
        args.add_options([
            ("log-level", None, None),
            ("log-color", None, None),
            ("log-file", None, None),
            ("log-filter", None, None)
        ])
        .unwrap();
    }

    fn load_args(&mut self, args: &Args) {
        #[rustfmt::skip]
        self.extend(
            [
                ("level", args.get("log-level")),
                ("color", args.get("log-color")),
                ("file", args.get("log-file")),
                ("filter", args.get("log-filter")),
            ]
            .iter()
            .map(convert::tuple_result_option_str)
        );
        if self.file.eq(&Some(String::default())) {
            self.file = None;
        }
        if args.get("debug").unwrap_or_default().is_some()
            && self.level < LevelFilter::Debug
        {
            self.level = LevelFilter::Debug;
        }
    }
}

impl LoadDirs for LogConfig {
    fn load_dirs(&mut self, dirs: &Dirs) {
        if self.file.eq(&Some(String::default())) {
            self.file = None;
        }
        if dirs.log.is_empty() == false
            && let Some(file) = self.file.as_mut()
            && file.starts_with("/") == false
        {
            file.insert(0, '/');
            file.insert_str(0, dirs.log.trim_end_matches('/'));
        }
    }
}

impl LoadEnv for LogConfig {
    fn load_env(&mut self) {
        self.extend(
            [
                ("level", getenv("LOG_LEVEL")),
                ("file", getenv("LOG_FILE")),
                ("color", getenv("LOG_COLOR")),
                ("filter", getenv("LOG_FILTER"))
            ]
            .iter()
            .map(convert::tuple_option_str)
        );
        if self.file.eq(&Some(String::default())) {
            self.file = None;
        }
    }
}
