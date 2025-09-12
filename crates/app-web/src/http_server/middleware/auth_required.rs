use {
    crate::{WebConfig, http_server::ext::RequestExt},
    actix_http::header,
    actix_web::{
        Error, HttpResponse,
        dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
        error::ErrorForbidden
    },
    app_base::prelude::*,
    std::{
        future::{Future, Ready, ready},
        pin::Pin
    }
};

#[derive(Default)]
pub struct AuthRequired;

impl<S> Transform<S, ServiceRequest> for AuthRequired
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static
{
    type Error = Error;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    type InitError = ();
    type Response = ServiceResponse;
    type Transform = AuthRequiredMiddleware<S>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthRequiredMiddleware { service }))
    }
}

pub struct AuthRequiredMiddleware<S> {
    service: S
}

impl<S> Service<ServiceRequest> for AuthRequiredMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static
{
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    type Response = ServiceResponse;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Skip auth checking to "/api" requests
        if [req.request().path(), "/"].concat().starts_with("/api/") {
            return Box::pin(self.service.call(req));
        }

        let request = req.request().clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            Env::is_debug().then(|| log::trace!("URL: {}", request.path()));

            let current_user = request.current_user().await;
            let path = [request.path(), "/"].concat();
            let auth = &request.config::<WebConfig>().auth;

            for (_name, item) in auth.modules.iter() {
                if path.starts_with(&item.url)
                    && (item.login.is_none()
                        || path.starts_with(item.login.as_ref().unwrap()) == false)
                    && item.skip.iter().any(|s| path.starts_with(s)) == false
                {
                    if let Ok(ref current_user) = current_user {
                        if item.roles.is_empty() == false
                            && let Err(err) = current_user.is_granted(
                                &item
                                    .roles
                                    .iter()
                                    .map(String::as_str)
                                    .collect::<Vec<_>>()
                            )
                        {
                            let res = HttpResponse::from_error(err);
                            return Ok(ServiceResponse::new(request, res));
                        }
                    } else {
                        let res = match item.login.as_ref() {
                            Some(login) => {
                                let mut auth_url = login.clone();
                                let back_url = request
                                    .uri()
                                    .path_and_query()
                                    .map(|v| v.as_str())
                                    .unwrap_or(request.path());

                                if back_url.starts_with(&auth_url) == false {
                                    auth_url.push_str("?back_url=");
                                    auth_url.push_str(&urlencoding::encode(back_url));
                                }

                                HttpResponse::TemporaryRedirect()
                                    .insert_header((header::LOCATION, auth_url.as_str()))
                                    .finish()
                            },
                            None => ErrorForbidden("Access denied.").error_response()
                        };

                        return Ok(ServiceResponse::new(request, res));
                    }
                }
            }

            // !!! (: Important drop(request) before fut.await :) !!!
            drop(request);

            fut.await
        })
    }
}
