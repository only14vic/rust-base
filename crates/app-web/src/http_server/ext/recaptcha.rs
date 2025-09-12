use {
    super::RequestHeadExt,
    actix_web::HttpRequest,
    app_base::prelude::*,
    reqwest::{Client, Method, Url},
    serde::{Deserialize, Serialize},
    serde_json::Value,
    std::{
        collections::HashSet, env, ops::Deref, str::FromStr, sync::LazyLock,
        time::Duration
    },
    validator::ValidationError
};

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(5))
        .tcp_keepalive(None)
        .deflate(true)
        .gzip(true)
        .build()
        .unwrap()
});

#[derive(Debug, Clone, Serialize)]
pub struct Recaptcha {
    key: String,
    secret: String,
    action: String
}

impl Recaptcha {
    pub fn from_request(req: &HttpRequest) -> Option<Self> {
        if Env::is_test() {
            return None;
        }

        if env::var("RECAPTCHA_KEY").is_ok() && env::var("RECAPTCHA_SECRET").is_ok() {
            return Some(Self {
                key: env::var("RECAPTCHA_KEY")
                    .expect("Env variable RECAPTCHA_KEY is not defined."),
                secret: env::var("RECAPTCHA_SECRET")
                    .expect("Env variable RECAPTCHA_SECRET is not defined."),
                action: req.path().to_string()
            });
        }

        None
    }

    pub async fn validate(&self, captcha: &str, user_ip: Option<String>) -> Void {
        let mut url =
            Url::parse("https://www.google.com/recaptcha/api/siteverify").unwrap();

        url.query_pairs_mut().extend_pairs(&[
            ("secret", self.secret.to_string()),
            ("response", captcha.to_string())
        ]);

        if let Some(user_ip) = user_ip {
            url.query_pairs_mut().append_pair("remoteip", &user_ip);
        }

        let response = CLIENT
            .request(Method::GET, url)
            .fetch_mode_no_cors()
            .send()
            .await?
            .json::<RecaptchaResponse>()
            .await?;

        if response.success {
            match response.score {
                Some(score) if score >= 0.5 => return Ok(()),
                _ => ()
            }
        }

        Err(ValidationError::new_with_message(
            "captcha",
            "Invalid verification code.",
            &[]
        ))?
    }
}

#[derive(Debug, Deserialize)]
pub struct RecaptchaResponse {
    pub success: bool,
    pub score: Option<f32>,
    pub hostname: Option<String>,
    pub action: Option<String>,
    pub error_codes: Option<HashSet<String>>
}

pub trait TRecaptcha {}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CaptchaForm {
    #[serde(default, skip_serializing)]
    captcha: Option<String>
}

impl Deref for CaptchaForm {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.captcha
    }
}

impl FromStr for CaptchaForm {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self { captcha: Some(s.to_string()) })
    }
}

impl From<String> for CaptchaForm {
    fn from(value: String) -> Self {
        Self { captcha: Some(value) }
    }
}

impl TryFrom<&Value> for CaptchaForm {
    type Error = &'static str;

    fn try_from(value: &Value) -> std::result::Result<Self, Self::Error> {
        match value.get("captcha") {
            Some(captcha) => {
                match captcha.as_str() {
                    Some(captcha) => Self::from_str(captcha),
                    None => Err("Не указан проверочный код.")
                }
            },
            None => Err("Не указан проверочный код.")
        }
    }
}

impl CaptchaForm {
    pub async fn validate(&self, req: &HttpRequest) -> Void {
        let recaptcha = req.recaptcha();

        if recaptcha.is_none() {
            return Ok(());
        }

        if self.captcha.is_none() {
            Err(ValidationError::new_with_message(
                "captcha",
                "Введите проверочный код.",
                &[]
            ))?
        }

        recaptcha
            .unwrap()
            .validate(
                self.captcha.as_ref().unwrap(),
                req.head().remote_ip().map(|v| v.to_string())
            )
            .await?;

        Ok(())
    }
}
