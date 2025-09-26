use {
    crate::{MigrationRow, Migrator},
    app_base::prelude::*,
    sqlx::{Postgres, migrate::Migration, prelude::*},
    std::collections::HashMap
};

pub trait MigrationQueries {
    async fn apply_search_path(&mut self) -> Void;

    async fn remove_invalid_migrations(&mut self, migrations: &[Migration]) -> Void;

    async fn get_installed(&mut self) -> Ok<HashMap<String, MigrationRow>>;

    async fn get_last_version(&mut self, offset: Option<usize>) -> Ok<Option<i64>>;

    async fn get_last_migration(&mut self) -> Ok<Option<MigrationRow>>;

    async fn get_remain_migrations(&mut self, up_version: i64) -> Ok<Vec<MigrationRow>>;
}

impl MigrationQueries for Migrator<'_, Postgres> {
    #[cold]
    async fn apply_search_path(&mut self) -> Void {
        if let Some(schema) = self.config.db_schema.as_ref() {
            sqlx::query(&format!("set search_path to {schema}"))
                .execute(self.db_conn().await?)
                .await?;
        }

        ok()
    }

    #[cold]
    async fn remove_invalid_migrations(&mut self, migrations: &[Migration]) -> Void {
        let installed = self.get_installed().await?;
        let db_conn = self.db_conn().await?;

        for (name, row) in installed {
            let m = migrations.iter().find(|m| m.description == name);

            if let Some(m) = m {
                sqlx::query(
                    r#"
                        delete from _sqlx_migrations
                        where version = $1 and checksum != $2
                    "#
                )
                .bind(row.version)
                .bind(m.checksum.as_ref())
                .execute(&mut *db_conn)
                .await?;
            } else {
                sqlx::query(
                    r#"
                        delete from _sqlx_migrations
                        where version = $1
                    "#
                )
                .bind(row.version)
                .execute(&mut *db_conn)
                .await?;
            }
        }

        ok()
    }

    #[cold]
    async fn get_installed(&mut self) -> Ok<HashMap<String, MigrationRow>> {
        self.db_conn()
            .await?
            .fetch_all(
                r#"
                    select *
                    from _sqlx_migrations
                    order by version
                "#
            )
            .await?
            .into_iter()
            .map(|row| {
                let r = MigrationRow::from_row(&row).unwrap();
                (r.description.to_owned(), r)
            })
            .collect::<HashMap<_, _>>()
            .into_ok()
    }

    #[cold]
    async fn get_last_version(&mut self, offset: Option<usize>) -> Ok<Option<i64>> {
        self.db_conn()
            .await?
            .fetch_optional(
                format!(
                    r#"
                        select version
                        from _sqlx_migrations
                        order by version desc
                        offset {offset}
                        limit 1
                    "#,
                    offset = offset.unwrap_or(0)
                )
                .as_str()
            )
            .await?
            .map(|row| row.get("version"))
            .into_ok()
    }

    #[cold]
    async fn get_last_migration(&mut self) -> Ok<Option<MigrationRow>> {
        self.db_conn()
            .await?
            .fetch_optional(
                r#"
                    select *
                    from _sqlx_migrations
                    order by version desc
                    limit 1
                "#
            )
            .await?
            .map(|row| MigrationRow::from_row(&row).unwrap())
            .into_ok()
    }

    #[cold]
    async fn get_remain_migrations(&mut self, up_version: i64) -> Ok<Vec<MigrationRow>> {
        sqlx::query(
            r#"
                select *
                from _sqlx_migrations
                where version > $1
                order by version desc
            "#
        )
        .bind(up_version)
        .fetch_all(self.db_conn().await?)
        .await?
        .into_iter()
        .map(|row| MigrationRow::from_row(&row).unwrap())
        .collect::<Vec<_>>()
        .into_ok()
    }
}
