use {
    crate::*,
    app_async::db::{DbConfig, db_pool},
    app_base::prelude::*,
    core::{cmp::Ordering, ops::Deref},
    futures::executor::block_on,
    sqlx::{
        Acquire, Database, Executor, Transaction,
        migrate::{Migrate, MigrateDatabase, Migration, Migrator as SqlxMigrator}
    },
    std::{borrow::Cow, rc::Rc, sync::Arc}
};

pub struct Migrator<'a, D>
where
    Self: MigrationQueries,
    D: Database + MigrateDatabase,
    D::Connection: Migrate + MigratePhantom,
    &'a mut D::Connection: Executor<'a>
{
    pub config: Arc<MigratorConfig>,
    tx: Option<Transaction<'a, D>>
}

impl<'a, D> Drop for Migrator<'a, D>
where
    Self: MigrationQueries,
    D: Database + MigrateDatabase,
    D::Connection: Migrate + MigratePhantom,
    &'a mut D::Connection: Executor<'a>
{
    fn drop(&mut self) {
        block_on(async {
            if self.config.dry_run {
                self.rollback().await.unwrap();
            } else if let Some(tx) = self.tx.take() {
                tx.commit().await.unwrap();
            }
        });
    }
}

impl<'a, D> Migrator<'a, D>
where
    Self: MigrationQueries,
    D: Database + MigrateDatabase,
    D::Connection: Migrate + MigratePhantom,
    &'a mut D::Connection: Executor<'a>
{
    pub fn new<C>(config: &dyn AsRef<AppConfig<C>>) -> Self
    where
        C: MigratorConfigExt
    {
        let migrator_config = config.as_ref().get::<MigratorConfig>();
        Self { config: migrator_config.clone(), tx: None }
    }

    #[cold]
    pub async fn up(mut self, count: Option<u32>) -> Void {
        let res = async {
            let files = Sorter.sort(&self.config.dir, self.config.simple)?;

            let migrations = self.get_migrations(&files, false, count).await?;
            self.remove_invalid_migrations(&migrations).await?;

            if let Some(schema) = self.config.db_schema.clone() {
                self.apply_search_path(&schema).await?;
            }

            let migrator = self.get_migrator(migrations);
            let mut migrate = self.get_migrate().await?;

            migrator.run_direct(&mut migrate).await?;

            ok()
        }
        .await;

        if res.is_err() {
            self.rollback().await?;
        }

        res
    }

    #[cold]
    pub async fn down(mut self, count: Option<u32>) -> Void {
        let res = async {
            let files = Sorter.sort(&self.config.dir, true)?;

            let migrations = self.get_migrations(&files, true, Some(0)).await?;
            //self.remove_invalid_migrations(&migrations).await?;

            let migrator = self.get_migrator(migrations);
            let offset = count.unwrap_or(u32::MAX) as usize;
            let up_version = self.get_last_version(offset.into()).await?.unwrap_or(-1);
            let will_revert = self.get_remain_migrations(up_version).await?;

            let mut not_found = will_revert
                .iter()
                .filter(|a| migrator.version_exists(a.version) == false);

            if let Some(m) = not_found.next() {
                Err(format!(
                    "File not found for migration {v}: {n}",
                    n = &m.description,
                    v = &m.version
                ))?;
            }

            let migrate = self.get_migrate().await?;
            migrator.undo(migrate, up_version).await?;

            self.config.verbose.then(|| {
                will_revert.iter().for_each(|a| {
                    log::info!("Reverted {v}: {n} ", v = a.version, n = a.description)
                })
            });

            ok()
        }
        .await;

        if res.is_err() {
            self.rollback().await?;
        }

        res
    }

    #[cold]
    pub async fn status(mut self) -> Void {
        let res = async {
            let files = Sorter.sort(&self.config.dir, self.config.simple)?;
            println!("{}", self.printf_sorted_list(&files).await?);

            ok()
        }
        .await;

        if res.is_err() {
            self.rollback().await?;
        }

        res
    }

    pub async fn db_conn(&mut self) -> Ok<&mut D::Connection> {
        if self.tx.is_none() {
            Box::pin(self.init_conn()).await?;
        }

        Ok(self.tx.as_mut().unwrap().acquire().await?)
    }

    async fn init_conn(&mut self) -> Void {
        if D::database_exists(&self.config.db_url).await? == false {
            D::create_database(&self.config.db_url).await?;
        }

        let db_config: Arc<_> = DbConfig {
            url: self.config.db_url.clone(),
            schema: self.config.db_schema.clone(),
            ..Default::default()
        }
        .into();

        self.tx =
            Some(db_pool(Some(&db_config)).await?.begin().await? as Transaction<'a, D>);

        self.apply_search_path(&self.config.schema.clone()).await?;
        self.db_conn().await?.ensure_migrations_table().await?;

        ok()
    }

    async fn rollback(&mut self) -> Void {
        if let Some(tx) = self.tx.take() {
            self.config
                .verbose
                .then(|| log::warn!("Rollback SQL transaction"));

            tx.rollback().await?;
        }

        ok()
    }

    #[cold]
    async fn printf_sorted_list(&mut self, files: &[Rc<FileMeta>]) -> Ok<String> {
        let mut all: Vec<(_, _)> =
            files.iter().map(|a| (a.description(), None)).collect();

        let installed = self.get_installed().await?;

        installed.into_iter().for_each(|a| {
            if let Some(b) = all.iter_mut().find(|c| c.0 == a.0) {
                b.1 = Some(a.1.version);
            } else {
                all.push((Cow::from(a.0), Some(a.1.version)));
            }
        });

        all.sort_by(|a, b| {
            match (a.1, b.1) {
                (Some(av), Some(bv)) => av.cmp(&bv),
                (Some(..), None) => Ordering::Less,
                _ => Ordering::Greater
            }
        });

        let res = all
            .iter()
            .map(|a| {
                format!(
                    "{version:<5}: {}",
                    a.0,
                    version = match a.1 {
                        Some(v) => Cow::from(v.to_string()),
                        None if a.0.ends_with(FileMeta::PHANTOM_INDENT) =>
                            Cow::from("--@--"),
                        None => Cow::from("-----")
                    },
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(res)
    }

    #[cold]
    async fn get_migrations(
        &mut self,
        files: &[Rc<FileMeta>],
        version_order: bool,
        count_new: Option<u32>
    ) -> Ok<Vec<Migration>> {
        let installed = self.get_installed().await?;
        let mut migrations = Vec::new();
        let mut count_new = count_new.unwrap_or(u32::MAX);
        let mut last_version = installed
            .values()
            .max_by_key(|a| a.version)
            .map(|a| a.version)
            .unwrap_or(0);

        let mut files = files.iter().collect::<Vec<_>>();

        if version_order {
            files.sort_by(|a, b| {
                let ra = installed.get(&*a.description());
                let rb = installed.get(&*b.description());

                match (ra, rb) {
                    (Some(ra), Some(rb)) => ra.version.cmp(&rb.version),
                    (Some(..), None) => Ordering::Less,
                    _ => Ordering::Greater
                }
            });
        }

        for file_meta in files.iter() {
            let version =
                match installed.get(&*file_meta.description()).map(|r| r.version) {
                    Some(v) => v,
                    None => {
                        if count_new == 0 {
                            break;
                        }
                        count_new -= 1;
                        last_version += 1;
                        last_version
                    }
                };

            let mut m: (Migration, Migration) = (**file_meta).deref().try_into()?;

            m.0.version = version;
            migrations.push(m.0);

            m.1.version = version;
            migrations.push(m.1);
        }

        Ok(migrations)
    }

    fn get_migrator(&self, migrations: Vec<Migration>) -> SqlxMigrator {
        SqlxMigrator {
            migrations: Cow::from(migrations),
            ignore_missing: true,
            locking: false,
            no_tx: false
        }
    }

    async fn get_migrate(&mut self) -> Ok<MigrateWrapper<'_, D::Connection>> {
        let verbose = self.config.verbose;
        Ok(MigrateWrapper { conn: self.db_conn().await?, verbose })
    }
}
