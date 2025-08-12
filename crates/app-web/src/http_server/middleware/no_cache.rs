use {
    actix_http::header::{self, TryIntoHeaderValue},
    actix_web::{
        Error,
        body::BoxBody,
        dev::{ServiceRequest, ServiceResponse},
        middleware::Next
    }
};

pub async fn no_cache(req: ServiceRequest, next: Next<BoxBody>) -> Result<ServiceResponse, Error> {
    let mut res = next.call(req).await?;

    if res.headers().contains_key(header::CACHE_CONTROL) == false {
        res.headers_mut().insert(
            header::CACHE_CONTROL,
            "no-store, no-cache, max-age=0, must-revalidate, proxy-revalidate"
                .try_into_value()
                .unwrap()
        );
    }

    Ok(res)
}
