use {
    app_base::prelude::*,
    serde::{Deserialize, Serialize}
};

#[derive(Debug, Clone, Hash, Eq, PartialEq, ExtendFromIter, Serialize, Deserialize)]
pub struct DbConfig {
    pub url: String,
    pub schema: Option<String>,
    pub min_conn: u32,
    pub max_conn: u32,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: String::default(),
            schema: None,
            min_conn: 1,
            max_conn: 100,
            acquire_timeout: 3,
            idle_timeout: 10,
            max_lifetime: 30
        }
    }
}

impl LoadEnv for DbConfig {
    fn load_env(&mut self) -> Void {
        self.extend(
            [
                ("url", getenv("DATABASE_URL")),
                ("min_conn", getenv("DATABASE_MIN_CONNECTIONS")),
                ("max_conn", getenv("DATABASE_MAX_CONNECTIONS")),
                ("schema", getenv("DATABASE_SCHEMA"))
            ]
            .iter()
            .map(convert::tuple_option_str)
        );
        ok()
    }
}

impl LoadArgs for DbConfig {
    fn load_args(&mut self, args: &Args) -> Void {
        self.extend(
            [("url", args.get("db-url"))]
                .iter()
                .map(convert::tuple_option_option_str)
        );
        ok()
    }
}
