use {
    app_base::prelude::*,
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

impl LoadEnv for JwtConfig {
    fn load_env(&mut self) -> Void {
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
        ok()
    }
}

impl LoadArgs for JwtConfig {
    fn load_args(&mut self, args: &Args) -> Void {
        self.extend(
            [
                ("secret", args.get("jwt-secret")),
                ("issuer", args.get("jwt-issuer")),
                ("audience", args.get("jwt-audience")),
                (
                    "access_token_lifetime",
                    args.get("jwt-access-token-lifetime")
                ),
                (
                    "refresh_token_lifetime",
                    args.get("jwt-refresh-token-lifetime")
                )
            ]
            .iter()
            .map(convert::tuple_result_option_str)
        );
        ok()
    }
}
