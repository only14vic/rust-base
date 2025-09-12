use {
    crate::{WebConfig, ext::RequestExt},
    actix_http::header::{self, TryIntoHeaderValue},
    actix_web::{
        Error,
        body::BoxBody,
        dev::{ServiceRequest, ServiceResponse},
        middleware::Next
    }
};

pub async fn cache_control(
    req: ServiceRequest,
    next: Next<BoxBody>
) -> Result<ServiceResponse, Error> {
    let web_config = req.request().config::<WebConfig>().clone();
    let path = [req.path(), "/"].concat();
    let mut res = next.call(req).await?;

    if res.headers().contains_key(header::CACHE_CONTROL) {
        return Ok(res);
    }

    if let Some(cache_age) = web_config.static_cache {
        let static_path = web_config.static_path.trim_end_matches("/");
        if path.starts_with(static_path)
            && path.get(static_path.len()..=static_path.len()) == Some("/")
        {
            res.headers_mut().insert(
                header::CACHE_CONTROL,
                header::HeaderValue::from_str(&format!(
                    "public, max-age={cache_age}, s-maxage={cache_age}"
                ))
                .unwrap()
            );
            return Ok(res);
        }
    }

    res.headers_mut().insert(
        header::CACHE_CONTROL,
        "no-store, no-cache, max-age=0, must-revalidate, proxy-revalidate"
            .try_into_value()
            .unwrap()
    );

    Ok(res)
}
