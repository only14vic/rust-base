use {
    super::FileMeta,
    app_base::prelude::*,
    futures::future::BoxFuture,
    sqlx::{
        PgConnection,
        migrate::{Migrate, MigrateError, Migration},
        prelude::Connection
    }
};

pub trait MigratePhantom
where
    Self: Connection + Migrate
{
    fn skip_phantom<'e: 'm, 'm>(
        &'e mut self,
        migration: &'m Migration
    ) -> BoxFuture<'m, Result<(), MigrateError>>;
}

impl MigratePhantom for PgConnection {
    fn skip_phantom<'e: 'm, 'm>(
        &'e mut self,
        migration: &'m Migration
    ) -> BoxFuture<'m, Result<(), MigrateError>> {
        Box::pin(async move {
            if migration.description.ends_with(FileMeta::PHANTOM_INDENT) {
                sqlx::query("delete from _sqlx_migrations where version = $1")
                    .bind(migration.version)
                    .execute(self)
                    .await?;
            }

            ok()
        })
    }
}
