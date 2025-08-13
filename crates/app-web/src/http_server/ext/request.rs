use {
    crate::{
        HtmlRenderContext, WebConfig,
        ext::{CurrentUser, DbWeb, JwtToken}
    },
    actix_web::{
        FromRequest, HttpRequest,
        dev::{Payload, RequestHead},
        http::header
    },
    app_base::prelude::*,
    regex::Regex,
    sqlx::{Pool, Postgres},
    std::{
        borrow::Cow,
        sync::{Arc, LazyLock}
    }
};

pub trait RequestExt {
    fn base_config(&self) -> &BaseConfig;

    fn web_config(&self) -> &WebConfig;

    fn db_pool(&self) -> &Pool<Postgres>;

    fn db_web(&self) -> &DbWeb;

    async fn current_user(&self) -> Result<CurrentUser, actix_web::Error>;

    async fn jwt_token(&self) -> Result<JwtToken, actix_web::Error>;

    fn language(&self) -> Cow<'_, str>;

    fn locale(&self) -> Cow<'_, str>;

    async fn html_render_context(&self) -> Ok<HtmlRenderContext>;
}

impl RequestExt for HttpRequest {
    fn base_config(&self) -> &BaseConfig {
        self.app_data::<Arc<BaseConfig>>().unwrap().as_ref()
    }

    fn web_config(&self) -> &WebConfig {
        self.app_data::<Arc<WebConfig>>().unwrap().as_ref()
    }

    fn db_pool(&self) -> &Pool<Postgres> {
        self.app_data::<Arc<Pool<Postgres>>>().unwrap().as_ref()
    }

    fn db_web(&self) -> &DbWeb {
        self.app_data::<DbWeb>().unwrap()
    }

    async fn current_user(&self) -> Result<CurrentUser, actix_web::Error> {
        CurrentUser::from_request(self, &mut Payload::None).await
    }

    async fn jwt_token(&self) -> Result<JwtToken, actix_web::Error> {
        JwtToken::from_request(self, &mut Payload::None).await
    }

    fn language(&self) -> Cow<'_, str> {
        let config = self.base_config();
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
        let config = self.base_config();
        let language = self.language();

        match config.locales.get(language.as_ref()) {
            Some(Some(locale)) => locale.into(),
            _ => format!("{}_{}", &language, &language.to_uppercase()).into()
        }
    }

    async fn html_render_context(&self) -> Ok<HtmlRenderContext> {
        Ok(HtmlRenderContext::from_request(self, &mut Payload::None).await?)
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
            Regex::new("(?i)android|webos|iphone|ipad|ipod|blackberry|mobile|opera mini").unwrap()
        });

        match self.user_agent() {
            Some(ua) => MOBILE_REGEX.is_match(&ua),
            None => false
        }
    }
}
