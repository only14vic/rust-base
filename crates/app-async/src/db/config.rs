use app_base::prelude::*;

#[derive(Debug, Clone, Hash, Eq, PartialEq, SetFromIter)]
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
    fn load_env(&mut self) -> app_base::prelude::Ok<&mut Self> {
        self.set_from_iter(
            [
                ("url", getenv("DATABASE_URL")),
                ("min_conn", getenv("DATABASE_MIN_CONNECTIONS")),
                ("max_conn", getenv("DATABASE_MAX_CONNECTIONS")),
                ("schema", getenv("DATABASE_SCHEMA"))
            ]
            .iter()
            .map(|(k, v)| (*k, v.as_ref().map(String::as_str)))
        )?;
        self.into_ok()
    }
}

impl LoadArgs for DbConfig {
    fn load_args(&mut self, args: &Args) -> Ok<&mut Self> {
        #[rustfmt::skip]
        self.set_from_iter(
            [
                ("url", args.get("db-url")),
            ]
            .iter().map(|(k, v)| {(
                *k, v.unwrap_or(&None).as_ref().map(String::as_str)
            )})
        )?;
        self.into_ok()
    }
}
