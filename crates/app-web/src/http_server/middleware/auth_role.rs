use {
    crate::{WebConfig, http_server::ext::RequestExt},
    actix_web::{Error, dev::ServiceRequest},
    app_base::prelude::*
};

pub async fn auth_role_extract(req: &ServiceRequest) -> Result<Vec<String>, Error> {
    Env::is_debug().then(|| log::trace!("URL: {}", req.path()));

    let mut roles = vec![];
    let api_path = req.request().config::<WebConfig>().api.path.to_string() + "/";

    // Don't extract roles for "/api" requests
    if [req.path(), "/"].concat().starts_with(&api_path) {
        return Ok(roles);
    }

    if let Ok(user) = req.request().current_user().await {
        roles.extend(user.role_idents());
    }

    Ok(roles)
}
