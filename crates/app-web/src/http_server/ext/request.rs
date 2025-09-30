use {
    crate::{
        HtmlRender, HtmlRenderContext,
        ext::{CurrentUser, DbWeb, Http, JwtToken}
    },
    actix_web::{
        FromRequest, HttpMessage, HttpRequest, HttpResponse,
        dev::{Payload, RequestHead},
        http::header
    },
    app_async::db::DbConfigApp,
    app_base::{prelude::*, type_name_simple},
    regex::Regex,
    sqlx::{Pool, Postgres},
    std::{
        borrow::Cow,
        rc::Rc,
        sync::{Arc, LazyLock}
    }
};

pub trait RequestExt {
    fn app<C: AppConfigExt>(&self) -> &App<C>;

    fn config<C: AppConfigExt>(&self) -> &Arc<C>;

    fn di(&self) -> &Di;

    fn db_pool(&self) -> &Arc<Pool<Postgres>>;

    fn db_web(&self) -> Rc<DbWeb>;

    fn db_config(&self) -> &Arc<DbConfigApp>;

    fn language(&self) -> Cow<'_, str>;

    fn locale(&self) -> Cow<'_, str>;

    async fn current_user(&self) -> Result<CurrentUser, actix_web::Error>;

    async fn jwt_token(&self) -> Result<JwtToken, actix_web::Error>;

    async fn html_render(&self) -> Http<HttpResponse>;

    async fn html_render_context(&self) -> Http<HtmlRenderContext>;
}

impl RequestExt for HttpRequest {
    fn app<C: AppConfigExt>(&self) -> &App<C> {
        self.app_data::<&App<C>>()
            .ok_or_else(|| {
                format!(
                    "There is no item HttpRequest::app_data::<&App<{}>>()",
                    type_name_simple!(C)
                )
            })
            .unwrap()
    }

    fn config<C: AppConfigExt>(&self) -> &Arc<C> {
        self.app_data::<Arc<C>>()
            .ok_or_else(|| {
                format!(
                    "There is no item HttpRequest::app_data::<Arc<{}>>()",
                    type_name_simple!(C)
                )
            })
            .unwrap()
    }

    fn di(&self) -> &Di {
        self.app_data::<&Di>()
            .ok_or("There is no item HttpRequest::app_data::<&Di>()")
            .unwrap()
    }

    fn db_pool(&self) -> &Arc<Pool<Postgres>> {
        self.app_data::<Arc<Pool<Postgres>>>()
            .ok_or("There is no item HttpRequest::app_data::<Arc<Pool<Postgres>>>()")
            .unwrap()
    }

    fn db_web(&self) -> Rc<DbWeb> {
        if self.extensions().contains::<Rc<DbWeb>>() == false {
            self.extensions_mut()
                .insert(Rc::new(DbWeb::new(self.db_pool())));
        }
        self.extensions().get::<Rc<DbWeb>>().unwrap().clone()
    }

    fn db_config(&self) -> &Arc<DbConfigApp> {
        self.app_data::<Arc<DbConfigApp>>()
            .ok_or("There is no item HttpRequest::app_data::<DbConfigApp>()")
            .unwrap()
    }

    fn language(&self) -> Cow<'_, str> {
        let config = self.config::<BaseConfig>();
        let default_lang = config.language.as_str();

        if let Some(cookie_lang) = self.cookie("lang")
            && config.locales.contains_key(cookie_lang.value())
        {
            return cookie_lang.value().to_owned().into();
        }

        if let Some(header) = self.headers().get(header::ACCEPT_LANGUAGE)
            && header.as_ref().len() >= 2
            && let Ok(header_lang) = str::from_utf8(&header.as_ref()[0..2])
            && config.locales.contains_key(header_lang)
        {
            return header_lang.into();
        }

        default_lang.into()
    }

    fn locale(&self) -> Cow<'_, str> {
        let config = self.config::<BaseConfig>();
        let language = self.language();

        match config.locales.get(language.as_ref()) {
            Some(Some(locale)) => locale.into(),
            _ => format!("{}_{}", &language, &language.to_uppercase()).into()
        }
    }

    async fn current_user(&self) -> Result<CurrentUser, actix_web::Error> {
        CurrentUser::from_request(self, &mut Payload::None).await
    }

    async fn jwt_token(&self) -> Result<JwtToken, actix_web::Error> {
        JwtToken::from_request(self, &mut Payload::None).await
    }

    async fn html_render(&self) -> Http<HttpResponse> {
        HtmlRender::from_request(self, &mut Payload::None)
            .await?
            .render_request(self)
            .await
    }

    async fn html_render_context(&self) -> Http<HtmlRenderContext> {
        HtmlRenderContext::from_request(self, &mut Payload::None).await
    }
}

pub trait RequestHeadExt {
    fn remote_ip(&self) -> Option<Cow<'_, str>>;

    fn user_agent(&self) -> Option<Cow<'_, str>>;

    fn is_mobile(&self) -> bool;
}

impl RequestHeadExt for RequestHead {
    fn remote_ip(&self) -> Option<Cow<'_, str>> {
        let ip = self
            .headers
            .get(header::X_FORWARDED_FOR)
            .map(|v| v.to_str().map(|v| v.into()).ok())
            .unwrap_or(self.peer_addr.map(|v| v.ip().to_string()));

        if let Some(ip) = ip {
            return ip
                .split_terminator(&[' ', ','])
                .next()
                .map(|s| s.to_string().into());
        }

        None
    }

    fn user_agent(&self) -> Option<Cow<'_, str>> {
        if let Some(v) = self.headers.get(header::USER_AGENT) {
            return v.to_str().map(|s| s.into()).ok();
        }

        None
    }

    fn is_mobile(&self) -> bool {
        static MOBILE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new("(?i)android|webos|iphone|ipad|ipod|blackberry|mobile|opera mini")
                .unwrap()
        });

        match self.user_agent() {
            Some(ua) => MOBILE_REGEX.is_match(&ua),
            None => false
        }
    }
}
