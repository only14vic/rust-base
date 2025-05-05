use {
    app_base::prelude::*,
    core::fmt::Display,
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

impl AppConfigExt for DbConfig {}

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

impl Iter<'_, (&'static str, String)> for DbConfig {
    fn iter(&self) -> impl Iterator<Item = (&'static str, String)> {
        [
            ("db.url", &self.url as &dyn Display),
            (
                "db.schema",
                Box::leak(Box::new(self.schema.as_deref().unwrap_or_default()))
            ),
            ("db.min_conn", &self.min_conn),
            ("db.max_conn", &self.max_conn),
            ("db.idle_timeout", &self.idle_timeout),
            ("db.max_lifetime", &self.max_lifetime),
            ("db.acquire_timeout", &self.acquire_timeout)
        ]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
    }
}

impl LoadArgs for DbConfig {
    fn init_args(&mut self, args: &mut Args) {
        args.add_options([
            ("db-url", None, None),
            ("db-schema", None, None),
            ("db-min-conn", None, None),
            ("db-max-conn", None, None),
            ("db-acquire-timeout", None, None)
        ])
        .unwrap();
    }

    fn load_args(&mut self, args: &Args) {
        self.extend(
            [
                ("url", args.get("db-url")),
                ("schema", args.get("db-schema")),
                ("min_conn", args.get("db-min-conn")),
                ("max_conn", args.get("db-max-conn")),
                ("acquire_timeout", args.get("db-acquire-timeout"))
            ]
            .iter()
            .map(convert::tuple_result_option_str)
        );
    }
}

impl LoadEnv for DbConfig {
    fn load_env(&mut self) {
        self.extend(
            [
                ("url", getenv("DATABASE_URL")),
                ("schema", getenv("DATABASE_SCHEMA")),
                ("min_conn", getenv("DATABASE_MIN_CONNECTIONS")),
                ("max_conn", getenv("DATABASE_MAX_CONNECTIONS")),
                ("acquire_timeout", getenv("DATABASE_CONNECTION_TIMEOUT"))
            ]
            .iter()
            .map(convert::tuple_option_str)
        );
    }
}

impl LoadDirs for DbConfig {
    fn load_dirs<'a>(&'a mut self, _dirs: &'a Dirs) {}
}
