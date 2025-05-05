use {
    crate::ext::{JwtToken, User},
    actix_web::error::ErrorUnauthorized,
    app_base::prelude::*,
    sqlx::{Acquire, Pool, Postgres},
    std::sync::Arc,
    uuid::Uuid
};

pub struct DbWeb {
    db_pool: Arc<Pool<Postgres>>
}

impl DbWeb {
    pub fn new(db_pool: &Arc<Pool<Postgres>>) -> Self {
        Self { db_pool: db_pool.clone() }
    }

    pub async fn find_user_by_id(&self, id: &Uuid) -> Ok<Option<User>> {
        Env::is_debug().then(|| log::trace!("SQL SELECT: app.users_view"));

        sqlx::query_as("select * from app.users_view where id = $1")
            .bind(id)
            .fetch_optional(self.db_pool.acquire().await?.acquire().await?)
            .await?
            .into_ok()
    }

    pub async fn find_user_by_login(&self, login: &str) -> Ok<Option<User>> {
        Env::is_debug().then(|| log::trace!("SQL SELECT: auth.find_user_by_login()"));

        sqlx::query_as(
            "select * from app.users_view where id = (auth.find_user_by_login($1)).id"
        )
        .bind(login)
        .fetch_optional(self.db_pool.acquire().await?.acquire().await?)
        .await?
        .into_ok()
    }

    pub async fn find_user_by_token(
        &self,
        token: &str,
        user_agent: &str
    ) -> Ok<Option<User>> {
        Env::is_debug().then(|| log::trace!("SQL SELECT: auth.find_user_by_token()"));

        sqlx::query_as("select * from auth.find_user_by_token($1, $2)")
            .bind(token)
            .bind(user_agent)
            .fetch_optional(self.db_pool.acquire().await?.acquire().await?)
            .await?
            .into_ok()
    }

    pub async fn find_user_by_jwt_token(&self, jwt: &JwtToken) -> Ok<User> {
        if let Some(user) = self.find_user_by_id(jwt.user_id()).await? {
            user.is_active()?;
            Ok(user)
        } else {
            Err(ErrorUnauthorized("User not found."))?
        }
    }
}
