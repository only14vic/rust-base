use {
    app_async::{TokioConfig, db::DbConfig},
    app_base::prelude::*,
    core::fmt::Display,
    serde::{Deserialize, Serialize},
    std::sync::Arc
};

pub trait MigratorConfigExt:
    AppConfigExt + AsRef<Arc<MigratorConfig>> + AsRef<Arc<TokioConfig>> + AsRef<Arc<DbConfig>>
{
}

#[derive(Debug, ExtendFromIter, Serialize, Deserialize)]
pub struct MigratorConfig {
    pub dir: String,
    pub dry_run: bool,
    pub simple: bool,
    pub verbose: bool,
    quiet: Option<bool>
}

impl AppConfigExt for MigratorConfig {
    const COMMAND: &str = "migrator";
}

impl Default for MigratorConfig {
    fn default() -> Self {
        Self {
            dir: "migrations".into(),
            dry_run: false,
            simple: false,
            verbose: true,
            quiet: None
        }
    }
}

impl Iter<'_, (&'static str, String)> for MigratorConfig {
    fn iter(&self) -> impl Iterator<Item = (&'static str, String)> {
        [
            ("migrator.dir", &self.dir as &dyn Display),
            ("migrator.dry_run", &self.dry_run),
            ("migrator.simple", &self.simple),
            ("migrator.verbose", &self.verbose)
        ]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
    }
}

impl LoadArgs for MigratorConfig {
    fn init_args(&mut self, args: &mut Args) {
        if args.get("command").unwrap() == Some(Self::COMMAND) {
            args.add_options([
                ("dir", "-d".into(), None),
                ("dry-run:b", "-n".into(), None),
                ("simple:b", "-s".into(), None),
                ("verbose:b", "-v".into(), None),
                ("quiet:b", "-q".into(), None)
            ])
            .unwrap();
        }
    }

    fn load_args(&mut self, args: &Args) {
        if args.get("command").unwrap() == Some(Self::COMMAND) {
            self.extend(
                [
                    ("dir", args.get("dir")),
                    ("dry_run", args.get("dry-run")),
                    ("simple", args.get("simple")),
                    ("verbose", args.get("verbose")),
                    ("quiet", args.get("quiet"))
                ]
                .iter()
                .map(convert::tuple_result_option_str)
            );

            if let Some(quiet) = self.quiet {
                self.verbose = !quiet;
            }
        }
    }
}

impl LoadDirs for MigratorConfig {
    fn load_dirs(&mut self, dirs: &Dirs) {
        if dirs.data.is_empty() == false && self.dir.starts_with("/") == false {
            self.dir.insert(0, '/');
            self.dir.insert_str(0, &dirs.data);
        }
    }
}

impl LoadEnv for MigratorConfig {
    fn load_env(&mut self) {
        #[rustfmt::skip]
        self.extend(
            [
                ("dir", getenv("MIGRATIONS_DIR")),
            ]
            .iter()
            .map(convert::tuple_option_str)
        );
    }
}
