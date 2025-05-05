use {
    super::MigratePhantom,
    futures::{future, future::BoxFuture},
    sqlx::{
        Database, Transaction,
        migrate::{AppliedMigration, Migrate, MigrateError, Migration},
        prelude::{Acquire, Connection}
    },
    std::{
        ops::{Deref, DerefMut},
        time::Duration
    }
};

pub struct MigrateWrapper<'a, C>
where
    C: Connection + Migrate + MigratePhantom,
    C::Database: Database<Connection = C>
{
    pub conn: &'a mut C,
    pub verbose: bool
}

impl<C> Migrate for MigrateWrapper<'_, C>
where
    C: Connection + Migrate + MigratePhantom,
    C::Database: Database<Connection = C>
{
    fn ensure_migrations_table(&mut self) -> BoxFuture<'_, Result<(), MigrateError>> {
        self.conn.ensure_migrations_table()
    }

    fn dirty_version(&mut self) -> BoxFuture<'_, Result<Option<i64>, MigrateError>> {
        self.conn.dirty_version()
    }

    fn list_applied_migrations(
        &mut self
    ) -> BoxFuture<'_, Result<Vec<AppliedMigration>, MigrateError>> {
        self.conn.list_applied_migrations()
    }

    fn lock(&mut self) -> BoxFuture<'_, Result<(), MigrateError>> {
        self.conn.lock()
    }

    fn unlock(&mut self) -> BoxFuture<'_, Result<(), MigrateError>> {
        self.conn.unlock()
    }

    fn apply<'e: 'm, 'm>(
        &'e mut self,
        migration: &'m Migration
    ) -> BoxFuture<'m, Result<std::time::Duration, MigrateError>> {
        Box::pin(async move {
            self.verbose.then(|| {
                log::info!(
                    "Applying {v}: {n}",
                    n = &migration.description,
                    v = &migration.version
                )
            });

            let res = self.conn.apply(migration).await?;
            self.conn.skip_phantom(migration).await?;

            Ok(res)
        })
    }

    #[allow(unreachable_code)]
    #[allow(unused_variables)]
    fn revert<'e: 'm, 'm>(
        &'e mut self,
        migration: &'m Migration
    ) -> BoxFuture<'m, Result<Duration, MigrateError>> {
        unreachable!();

        let fut = self.conn.revert(migration);
        Box::pin(async move {
            let res = fut.await?;
            self.verbose.then(|| {
                log::info!(
                    "Reverted {v}: {n}",
                    n = &migration.description,
                    v = &migration.version
                )
            });
            Ok(res)
        })
    }
}

impl<'a, C> Acquire<'a> for MigrateWrapper<'a, C>
where
    C: Connection + Migrate + MigratePhantom,
    C::Database: Database<Connection = C>
{
    type Database = C::Database;

    type Connection = Self;

    fn acquire(self) -> BoxFuture<'a, Result<Self::Connection, sqlx::Error>> {
        Box::pin(future::ok(self))
    }

    fn begin(
        self
    ) -> BoxFuture<'a, Result<Transaction<'a, Self::Database>, sqlx::Error>> {
        Box::pin(self.conn.begin())
    }
}

impl<C> Deref for MigrateWrapper<'_, C>
where
    C: Connection + Migrate + MigratePhantom,
    C::Database: Database<Connection = C>
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.conn
    }
}

impl<C> DerefMut for MigrateWrapper<'_, C>
where
    C: Connection + Migrate + MigratePhantom,
    C::Database: Database<Connection = C>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.conn
    }
}
