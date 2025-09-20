use {
    super::JwtEncoder,
    accessory::Accessors,
    actix::fut,
    actix_http::header,
    actix_web::{
        FromRequest, HttpMessage, HttpRequest,
        dev::Payload,
        error::{ErrorBadRequest, ErrorUnauthorized}
    },
    app_base::prelude::*,
    serde::{Deserialize, Serialize},
    std::{borrow::BorrowMut, future::Future, pin::Pin, sync::Arc},
    uuid::Uuid
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Accessors)]
#[access(get)]
pub struct JwtToken {
    #[serde(skip)]
    token: String,
    iss: String,
    aud: String,
    exp: usize,
    chk: String,
    role: String,
    user_id: Uuid
}

impl JwtToken {
    pub fn with_token(mut self, token: &str) -> Self {
        self.token = token.to_string();
        self
    }

    pub fn token_hash(&self) -> String {
        format!("{:x}", md5::compute(&self.token))
    }
}

impl FromRequest for JwtToken {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        if let Some(res) = req.extensions().get::<Result<Self, String>>() {
            return match res {
                Ok(token) => Box::pin(fut::ok(token.clone())),
                Err(err) => Box::pin(fut::err(ErrorUnauthorized(err.to_string())))
            };
        }

        let req = req.clone();

        Box::pin(async move {
            Env::is_debug().then(|| log::trace!("URL: {}", req.path()));

            async {
                let authorization = req
                    .headers()
                    .get(header::AUTHORIZATION)
                    .map(|h| String::from_utf8(h.as_bytes().to_vec()))
                    .ok_or(ErrorUnauthorized("User is not authorized."))?
                    .map_err(ErrorBadRequest)?
                    .trim()
                    .to_string();

                let token = authorization
                    .get(authorization.rfind(' ').map(|i| i + 1).unwrap_or(0)..)
                    .unwrap();

                let encoder = req
                    .app_data::<Arc<JwtEncoder>>()
                    .expect("JwtEncoder does not exist in request.app_data()");

                let token = encoder
                    .decode_token(token)
                    .map_err(|_| ErrorUnauthorized("Invalid token."))?;

                Ok(token)
            }
            .await
            .inspect(|token| {
                req.extensions_mut()
                    .borrow_mut()
                    .insert(Ok(token.clone()) as Result<Self, String>);
            })
            .inspect_err(|err: &Self::Error| {
                req.extensions_mut()
                    .borrow_mut()
                    .insert(Err(err.to_string()) as Result<Self, String>);
            })
        })
    }
}
