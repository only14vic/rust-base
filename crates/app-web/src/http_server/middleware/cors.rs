use {
    crate::{WebConfig, ext::RequestHeadExt},
    actix_cors::Cors,
    std::sync::Arc
};

#[allow(dead_code)]
pub fn cors(web_config: &Arc<WebConfig>) -> Cors {
    let web_config = web_config.clone();
    Cors::default()
        .allowed_origin_fn(move |header, head| {
            if head
                .remote_ip()
                .map(|ip| ["127.0.0.1", "::1"].contains(&ip.as_ref()))
                == Some(true)
            {
                return true;
            }

            if let Ok(origin) = header.to_str() {
                origin.ends_with(&("://".to_owned() + &web_config.host))
            } else {
                false
            }
        })
        .block_on_origin_mismatch(true)
        .max_age(3600)
}
