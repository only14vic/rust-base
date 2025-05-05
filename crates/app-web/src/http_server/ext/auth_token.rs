use {
    accessory::Accessors,
    serde::{Deserialize, Serialize},
    std::borrow::Cow
};

#[derive(Debug, Serialize, Deserialize, Accessors)]
#[access(get)]
pub struct AuthToken<'a> {
    access_token: Cow<'a, str>,
    refresh_token: Cow<'a, str>
}

impl<'a> AuthToken<'a> {
    pub fn new(access_token: &'a str, refresh_token: &'a str) -> Self {
        Self {
            access_token: Cow::Borrowed(access_token),
            refresh_token: Cow::Borrowed(refresh_token)
        }
    }
}
