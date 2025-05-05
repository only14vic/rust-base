use {
    super::JwtToken,
    crate::http_server::ext::JwtConfig,
    app_base::prelude::*,
    jsonwebtoken::{
        Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode
    },
    serde_json::Value as JsonValue,
    uuid::Uuid
};
pub use jsonwebtoken::errors::{
    Error as JwtError, ErrorKind as JwtErrorKind, Result as JwtResult
};

pub struct JwtEncoder {
    validation: Validation,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey
}

impl JwtEncoder {
    pub fn new(config: &JwtConfig) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);

        validation.set_audience(&[&config.audience]);
        validation.set_issuer(&[&config.issuer]);
        validation
            .set_required_spec_claims(&["aud", "iss", "exp", "user_id", "role", "chk"]);

        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());

        Self { validation, encoding_key, decoding_key }
    }

    pub fn encode_token(&self, jwt: &JwtToken) -> JwtResult<String> {
        encode(&Header::default(), &jwt, &self.encoding_key)
    }

    pub fn decode_token(&self, token: &str) -> JwtResult<JwtToken> {
        let token = token.trim();
        let token = token
            .get(token.rfind(' ').map(|i| i + 1).unwrap_or(0)..)
            .unwrap();

        decode::<JwtToken>(token, &self.decoding_key, &self.validation)
            .map(|res| res.claims.with_token(token))
    }
}

pub fn jwt_base64_decode(token: &str) -> Ok<(JsonValue, JsonValue)> {
    let token = token.get(token.rfind(' ').unwrap_or(0)..).unwrap().trim();

    let mut parts: Vec<JsonValue> = token
        .split('.')
        .map_while(|s| filters::base64_decode(s, false).ok())
        .filter_map(|s| serde_json::from_str(&s).ok())
        .collect();

    if parts.len() < 2 {
        Err("Invalid JWT format.")?
    }

    (parts.remove(0), parts.remove(0)).into_ok()
}

pub fn jwt_user_id(token: &str) -> Ok<Uuid> {
    let (.., jwt) = jwt_base64_decode(token)?;
    jwt.as_object()
        .ok_or("JWT invalid (0)")?
        .get("user_id")
        .ok_or("JWT invalid (1)")?
        .as_str()
        .ok_or("JWT invalid (2)")?
        .parse::<Uuid>()
        .map_err(|_| "JWT invalid (3)")?
        .into_ok()
}
