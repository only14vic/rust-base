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
pub struct ErrHttp(pub Box<dyn Error>);

pub type Http<T> = Result<T, ErrHttp>;

pub type VoidHttp = Result<(), ErrHttp>;

impl ErrHttp {
    pub fn new(error: Box<dyn Error>) -> Self {
        match error.downcast::<Box<ErrHttp>>() {
            Ok(e) => **e,
            Err(e) => {
                match e.downcast::<Box<Err>>() {
                    Ok(e) => ErrHttp::new(e.0),
                    Err(e) => {
                        match e.downcast::<Box<ErrAsync>>() {
                            Ok(e) => ErrHttp::new(e.0),
                            Err(e) => ErrHttp(e)
                        }
                    },
                }
            },
        }
    }
}

impl Error for Box<ErrHttp> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.deref())
    }
}

impl Deref for ErrHttp {
    type Target = dyn Error;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Display for ErrHttp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E: Into<Box<dyn Error>>> From<E> for ErrHttp {
    #[inline(always)]
    fn from(value: E) -> Self {
        Self::new(value.into())
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
