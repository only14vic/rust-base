use {
    //super::CaptchaForm,
    app_base::serde::*,
    serde::{Deserialize, Serialize},
    std::ops::Not,
    validator::Validate
};

#[non_exhaustive]
#[derive(Debug, Default, Deserialize, Serialize, Validate)]
#[serde(default)]
pub struct AuthForm {
    #[validate(required(message = "Required."))]
    #[validate(length(max = 100, message = "Too long."))]
    #[serde(deserialize_with = "skip_empty_string_trim")]
    pub login: Option<String>,

    #[validate(required(message = "Required."))]
    #[validate(length(max = 100, message = "Too long."))]
    #[serde(deserialize_with = "skip_empty_string")]
    pub password: Option<String> /* pub captcha: CaptchaForm */
}

impl AuthForm {
    pub fn new(login: &str, password: &str) -> Self {
        Self {
            login: login.trim().is_empty().not().then_some(login.into()),
            password: password.is_empty().not().then_some(password.into()),
            ..Default::default()
        }
    }
}
