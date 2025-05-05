use {
    actix_http::header::{self, HeaderValue},
    actix_web::{
        Error,
        body::MessageBody,
        dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready}
    },
    app_base::prelude::*,
    std::{
        future::{Future, Ready, ready},
        pin::Pin
    }
};

#[derive(Default)]
pub struct AuthHeader;

impl<S, B> Transform<S, ServiceRequest> for AuthHeader
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static
{
    type Error = Error;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    type InitError = ();
    type Response = ServiceResponse<B>;
    type Transform = AuthHeaderMiddleware<S>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthHeaderMiddleware { service }))
    }
}

pub struct AuthHeaderMiddleware<S> {
    service: S
}

impl<S, B> Service<ServiceRequest> for AuthHeaderMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody + 'static,
    S::Future: 'static
{
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    type Response = ServiceResponse<B>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        Env::is_debug().then(|| log::trace!("URL: {}", req.path()));

        let mut authorization = None;

        if let Ok(cookies) = req.cookies()
            && req.headers().contains_key(header::AUTHORIZATION) == false
        {
            if let Some(token) = cookies.iter().find(|c| c.name() == "access_token") {
                authorization = Some(token.value().to_string());
            } else if let Some(token) =
                cookies.iter().find(|c| c.name() == "refresh_token")
            {
                authorization = Some(token.value().to_string());
            }
        }

        if let Some(authorization) = authorization {
            req.headers_mut().insert(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {authorization}")).unwrap()
            );
        }

        let fut = self.service.call(req);

        return Box::pin(fut);
    }
}
