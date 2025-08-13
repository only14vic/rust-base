use {
    actix_web::{HttpResponse, ResponseError, body::BoxBody, http::StatusCode},
    app_base::prelude::*,
    core::ops::Deref,
    std::{
        error::Error,
        fmt::{Display, Formatter}
    },
    validator::{ValidationError, ValidationErrors}
};

#[derive(Debug)]
pub struct ErrHttp(pub Err);

pub type OkHttp = Result<HttpResponse, ErrHttp>;

impl<E: Into<Box<dyn Error>>> From<E> for ErrHttp {
    #[inline(always)]
    fn from(value: E) -> Self {
        Self(Err::new(value.into()))
    }
}

impl Into<Err> for ErrHttp {
    fn into(self) -> Err {
        self.0
    }
}

impl Deref for ErrHttp {
    type Target = Err;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ErrHttp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ResponseError for ErrHttp {
    fn status_code(&self) -> StatusCode {
        self.error_response().status()
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        if let Some(e) = self.downcast_ref::<actix_web::Error>() {
            e.error_response()
        } else if let Some(e) = self.downcast_ref::<ValidationErrors>() {
            HttpResponse::BadRequest().body(e.to_string())
        } else if let Some(e) = self.downcast_ref::<ValidationError>() {
            HttpResponse::BadRequest().body(e.to_string())
        } else {
            HttpResponse::InternalServerError().body(self.to_string())
        }
    }
}
