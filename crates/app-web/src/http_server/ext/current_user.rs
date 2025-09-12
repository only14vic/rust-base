use {
    super::{RequestExt, RequestHeadExt},
    crate::WebConfig,
    accessory::Accessors,
    actix::fut,
    actix_web::{
        FromRequest, HttpMessage, HttpRequest,
        dev::Payload,
        error::{ErrorForbidden, ErrorUnauthorized}
    },
    app_async::cache::*,
    app_base::prelude::*,
    chrono::{DateTime, FixedOffset, NaiveDate},
    serde::{Deserialize, Serialize},
    sqlx::{postgres::PgRow, prelude::*},
    std::{
        borrow::BorrowMut, collections::HashSet, fmt::Display, future::Future,
        ops::Deref, pin::Pin, rc::Rc, sync::Arc
    },
    uuid::Uuid
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    #[serde(flatten)]
    user: Rc<User>
}

impl From<&Rc<User>> for CurrentUser {
    fn from(user: &Rc<User>) -> Self {
        Self { user: Rc::clone(user) }
    }
}

impl FromRow<'_, PgRow> for CurrentUser {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self { user: Rc::new(User::from_row(row)?) })
    }
}

impl Display for CurrentUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.username)
    }
}

impl Deref for CurrentUser {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        self.user.as_ref()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Accessors, FromRow)]
#[access(get)]
#[serde(default)]
#[sqlx(default)]
pub struct User {
    id: Uuid,
    avatar_id: Option<Uuid>,
    #[access(get(ty(&str)))]
    role: String,
    roles: Vec<String>,
    #[access(get(ty(&str)))]
    username: String,
    #[access(get(ty(&str)))]
    email: String,
    #[access(get(ty(&str)))]
    phone: String,
    #[access(get(ty(&str)))]
    first_name: String,
    #[access(get(ty(&str)))]
    second_name: String,
    #[access(get(ty(&str)))]
    last_name: String,
    #[access(get(ty(&str)))]
    full_name: String,
    #[access(get(ty(&str)))]
    short_name: String,
    birthday: Option<NaiveDate>,
    about: Option<String>,
    confirmed_at: Option<DateTime<FixedOffset>>,
    blocked_at: Option<DateTime<FixedOffset>>,
    deleted_at: Option<DateTime<FixedOffset>>,
    updated_at: DateTime<FixedOffset>,
    created_at: DateTime<FixedOffset>
}

impl User {
    pub fn role_idents(&self) -> HashSet<String> {
        let mut idents = self
            .roles
            .iter()
            .map(|s| format!("ROLE_{}", s.to_uppercase()))
            .collect::<HashSet<_>>();

        idents.insert(format!("ROLE_{}", &self.role.to_uppercase()));

        idents
    }

    pub fn is_active(&self) -> Result<(), actix_web::Error> {
        if self.blocked_at.is_some() {
            return Err(ErrorForbidden("User is blocked."))?;
        }

        if self.deleted_at.is_some() {
            return Err(ErrorForbidden("User not found."))?;
        }

        if self.confirmed_at.is_none() {
            return Err(ErrorForbidden("User is not confirmed."))?;
        }

        Ok(())
    }

    pub fn is_granted(&self, roles: &[&str]) -> Result<(), actix_web::Error> {
        let has_role = roles.iter().any(|&role| {
            self.role.as_str() == role || self.roles.iter().any(|s| s.as_str() == role)
        });

        if has_role == false {
            return Err(ErrorForbidden("Access denied."))?;
        }

        Ok(())
    }
}

impl FromRequest for CurrentUser {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        if let Some(res) = req.extensions().get::<Result<Self, String>>() {
            return match res {
                Ok(user) => Box::pin(fut::ok(user.clone())),
                Err(err) => Box::pin(fut::err(ErrorUnauthorized(err.clone())))
            };
        }

        let req = req.clone();

        Box::pin(async move {
            Env::is_debug().then(|| log::trace!("URL: {}", req.path()));

            async {
                // Verify current JWT token
                let jwt = req.jwt_token().await?;

                let token_hash = jwt.token_hash();
                let token_lifetime =
                    req.config::<WebConfig>().jwt.access_token_lifetime as u64;
                let cacher = req.app_data::<Cacher<ArrayCache>>().unwrap();

                let c_user_id: Option<Arc<Uuid>> = cacher
                    .get(&["users_tokens", &token_hash])
                    .await
                    .unwrap_or(None);

                if c_user_id.as_deref() == Some(&Uuid::default()) {
                    return Err(ErrorUnauthorized("User not found."))?;
                }

                let c_user: Option<Arc<User>> = match &c_user_id {
                    Some(user_id) => {
                        cacher
                            .get(&["users", &user_id.to_string()])
                            .await
                            .unwrap_or(None)
                    },
                    None => None
                };

                let user: User = match c_user {
                    Some(user_cache) => user_cache.as_ref().clone(),
                    None => {
                        let user_agent = req.head().user_agent().unwrap_or_default();

                        match req
                            .db_web()
                            .find_user_by_token(jwt.token(), &user_agent)
                            .await
                        {
                            Ok(Some(user)) => {
                                if false == Env::is_test() {
                                    cacher
                                        .set(
                                            &["users", &user.id().to_string()],
                                            user.clone(),
                                            token_lifetime
                                        )
                                        .await
                                        .ok();
                                    cacher
                                        .set(
                                            &["users_tokens", &token_hash],
                                            *user.id(),
                                            token_lifetime
                                        )
                                        .await
                                        .ok();
                                }
                                user
                            },
                            Ok(None) => {
                                cacher
                                    .set(
                                        &["users_tokens", &token_hash],
                                        Uuid::default(),
                                        10
                                    )
                                    .await
                                    .ok();
                                return Err(ErrorUnauthorized("User not found."))?;
                            },
                            Err(err) => {
                                cacher
                                    .set(
                                        &["users_tokens", &token_hash],
                                        Uuid::default(),
                                        10
                                    )
                                    .await
                                    .ok();
                                return Err(ErrorUnauthorized(err.to_string()))?;
                            }
                        }
                    }
                };

                let current_user = CurrentUser::from(&user.into());
                current_user.is_active()?;

                Ok(current_user)
            }
            .await
            .inspect(|current_user| {
                req.extensions_mut()
                    .borrow_mut()
                    .insert(Ok(current_user.clone()) as Result<Self, String>);
            })
            .inspect_err(|err: &Self::Error| {
                req.extensions_mut()
                    .borrow_mut()
                    .insert(Err(err.to_string()) as Result<Self, String>);
            })
        })
    }
}
