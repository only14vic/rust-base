use {
    crate::{
        WebConfig,
        ext::ErrHttp,
        http_server::ext::{RequestExt, RequestHeadExt}
    },
    actix_web::{FromRequest, HttpMessage, HttpRequest, dev::Payload, web},
    app_base::prelude::*,
    chrono::Local,
    serde::Serialize,
    serde_json::{Value, json},
    std::{borrow::Cow, cell::RefCell, future::Future, ops::Deref, pin::Pin, rc::Rc},
    tera::Context as TeraContext
};

#[derive(Debug, Default, Clone)]
pub struct HtmlRenderContext(Rc<RefCell<TeraContext>>);

impl HtmlRenderContext {
    pub fn add(&self, key: &str, value: &impl Serialize) {
        self.0.borrow_mut().insert(key, &json!(value));
    }
}

impl Deref for HtmlRenderContext {
    type Target = RefCell<TeraContext>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for HtmlRenderContext {
    type Error = ErrHttp;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        if req.extensions().contains::<Self>() == false {
            Env::is_debug().then(|| log::trace!("URL: {}", req.path()));

            let context = Self::default();
            let base_config = req.config::<BaseConfig>();
            let web_config = req.config::<WebConfig>();
            let mut app = Value::Object(Default::default());

            app.as_object_mut().unwrap().extend([
                ("language".into(), req.language().into()),
                ("locale".into(), req.locale().into()),
                (
                    "locales".into(),
                    serde_json::Map::from_iter(
                        base_config.locales.iter().map(|(n, v)| {
                            (n.to_string(), v.to_json().unwrap_or_default())
                        })
                    )
                    .into()
                ),
                ("timezone".into(), base_config.timezone.as_str().into()),
                ("host".into(), web_config.host.as_str().into()),
                ("hostname".into(), web_config.hostname.as_str().into()),
                ("base_url".into(), web_config.base_url.as_str().into()),
                ("api_url".into(), web_config.api.url.as_str().into()),
                ("static_path".into(), web_config.static_path.as_str().into()),
                ("is_mobile".into(), req.head().is_mobile().into()),
                ("user".into(), ().into())
            ]);

            app.as_object_mut().unwrap().insert(
                "env".into(),
                Value::from_iter([
                    ("APP_DEBUG", Env::is_debug().to_json().unwrap()),
                    ("APP_ENV", Env::env().into())
                ])
            );

            app.as_object_mut().unwrap().insert(
                "query".into(),
                Value::from_iter(
                    [
                        web::Query::<Vec<(Cow<str>, Cow<str>)>>::from_query(
                            req.query_string()
                        )
                        .map(web::Query::into_inner)
                        .unwrap_or_default(),
                        req.match_info()
                            .iter()
                            .map(|(name, value)| (name.into(), value.into()))
                            .collect()
                    ]
                    .concat()
                )
            );

            app.as_object_mut().unwrap().insert(
                "req".into(),
                Value::from_iter([
                    ("url", req.path()),
                    ("method", req.method().as_str()),
                    ("path", req.match_pattern().as_deref().unwrap_or(req.path())),
                    (
                        "referer",
                        req.headers()
                            .get("referer")
                            .map(|v| v.to_str().unwrap_or_default())
                            .unwrap_or_default()
                    )
                ])
            );

            context.add("app", &app);
            //context.add("recaptcha", &req.recaptcha());

            context.add("time", &Local::now());

            req.extensions_mut().insert(context);
        }

        let req = req.clone();

        Box::pin(async move {
            if let Ok(user) = req.current_user().await {
                let mut extensions = req.extensions_mut();
                let context = extensions.get_mut::<Self>().unwrap();
                let mut app = context.borrow_mut().remove("app").unwrap();

                app.as_object_mut()
                    .unwrap()
                    .insert("user".into(), json!(user));

                context.borrow_mut().insert("app", &app);
            }

            req.extensions()
                .get::<Self>()
                .ok_or("HtmlRenderContext does not exist in request.")
                .unwrap()
                .clone()
                .into_ok()
        })
    }
}
