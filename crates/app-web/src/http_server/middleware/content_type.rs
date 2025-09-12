use {
    actix_http::header::{self, TryIntoHeaderValue},
    actix_web::{
        Error,
        body::BoxBody,
        dev::{ServiceRequest, ServiceResponse},
        http::header::ContentType,
        middleware::Next
    },
    futures::future::LocalBoxFuture
};

pub fn content_type(
    default: ContentType
) -> impl Fn(
    ServiceRequest,
    Next<BoxBody>
) -> LocalBoxFuture<'static, Result<ServiceResponse, Error>> {
    move |req, next| Box::pin(content_type_middleware(req, next, default.clone()))
}

async fn content_type_middleware(
    mut req: ServiceRequest,
    next: Next<BoxBody>,
    default: ContentType
) -> Result<ServiceResponse, Error> {
    let value = default.try_into_value().unwrap();

    if req.headers().get(header::ACCEPT).is_none() {
        req.headers_mut().insert(header::ACCEPT, value.clone());
    }

    if req.headers().get(header::CONTENT_TYPE).is_none() {
        req.headers_mut().insert(header::CONTENT_TYPE, value);
    }

    next.call(req).await
}
