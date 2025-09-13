use {
    app_base::prelude::*,
    core::fmt::Display,
    serde::{Deserialize, Serialize},
    std::fmt::Debug
};

#[derive(Debug, ExtendFromIter, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    pub audience: String,
    pub access_token_lifetime: usize,
    pub refresh_token_lifetime: usize
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "123456".into(),
            issuer: "My Company".into(),
            audience: "My Company".into(),
            access_token_lifetime: 300,
            refresh_token_lifetime: 604800
        }
    }
}

impl Iter<'_, (&'static str, String)> for JwtConfig {
    fn iter(&self) -> impl Iterator<Item = (&'static str, String)> {
        [
            ("web.jwt.secret", &self.secret as &dyn Display),
            ("web.jwt.issuer", &self.issuer),
            ("web.jwt.audience", &self.audience),
            ("web.jwt.access_token_lifetime", &self.access_token_lifetime),
            (
                "web.jwt.refresh_token_lifetime", &self.refresh_token_lifetime
            )
        ]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
    }
}

impl LoadArgs for JwtConfig {
    fn init_args(&mut self, args: &mut Args) {
        args.add_options([
            ("web-jwt-secret", None, None),
            ("web-jwt-issuer", None, None),
            ("web-jwt-audience", None, None),
            ("web-jwt-access-token-lifetime", None, None),
            ("web-jwt-refresh-token-lifetime", None, None)
        ])
        .unwrap();
    }

    fn load_args(&mut self, args: &Args) {
        self.extend(
            [
                ("secret", args.get("web-jwt-secret")),
                ("issuer", args.get("web-jwt-issuer")),
                ("audience", args.get("web-jwt-audience")),
                (
                    "access_token_lifetime",
                    args.get("web-jwt-access-token-lifetime")
                ),
                (
                    "refresh_token_lifetime",
                    args.get("web-jwt-refresh-token-lifetime")
                )
            ]
            .iter()
            .map(convert::tuple_result_option_str)
        );
    }
}

impl LoadEnv for JwtConfig {
    fn load_env(&mut self) {
        self.extend(
            [
                ("secret", getenv("JWT_SECRET")),
                ("issuer", getenv("JWT_ISSUER")),
                ("audience", getenv("JWT_AUDIENCE")),
                ("access_token_lifetime", getenv("JWT_ACCESS_TOKEN_LIFETIME")),
                (
                    "refresh_token_lifetime",
                    getenv("JWT_REFRESH_TOKEN_LIFETIME")
                )
            ]
            .iter()
            .map(convert::tuple_option_str)
        );
    }
}
