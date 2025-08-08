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

impl LogConfig {
    pub fn with_log_dir(&mut self, dir: &str) -> &mut Self {
        if dir.is_empty() == false {
            if let Some(file) = self.file.as_mut() {
                if file.starts_with("/") == false {
                    file.insert(0, '/');
                    file.insert_str(0, dir.trim_end_matches('/'));
                }
            }
        }
        self
    }
}

impl LoadEnv for LogConfig {
    fn load_env(&mut self) -> Void {
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
        ok()
    }
}

impl LoadArgs for LogConfig {
    fn load_args(&mut self, args: &Args) -> Void {
        #[rustfmt::skip]
        self.extend(
            [
                ("level", args.get("log-level")),
                ("color", args.get("log-color")),
                ("file", args.get("log-file")),
                ("filter", args.get("log-filter")),
            ]
            .iter()
            .map(convert::tuple_option_option_str)
        );
        ok()
    }
}
