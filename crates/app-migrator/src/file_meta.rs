use {
    super::NodeInfo,
    app_base::prelude::*,
    sqlx::migrate::{Migration, MigrationType},
    std::{borrow::Cow, cmp::Ordering, error::Error, fs::read_to_string, path::Path}
};

#[derive(Debug, PartialEq)]
pub struct FileMeta {
    pub file_path: Box<Path>,
    pub nodes: Vec<NodeInfo>
}

impl FileMeta {
    pub const PHANTOM_INDENT: &'static str = ".@";

    pub fn file_name(&self) -> &str {
        self.file_path.file_name().unwrap().to_str().unwrap()
    }

    pub fn description(&self) -> Cow<'_, str> {
        Cow::from(
            self.file_name()
                .replace(".up.sql", "")
                .replace(".down.sql", "")
                .replace(".sql", "")
        )
    }

    pub fn is_phantom(&self) -> bool {
        self.file_name().contains(Self::PHANTOM_INDENT)
    }

    pub fn try_new(file_path: &dyn AsRef<Path>) -> Ok<Self> {
        let file_path: Box<Path> = Path::new(file_path.as_ref()).into();
        let content = read_to_string(&file_path)?;
        let queries = pg_query::split_with_scanner(&content)
            .inspect_err(|_| eprintln!("Error in {}", file_path.to_string_lossy()))?;
        let mut nodes = vec![];

        for query in queries {
            let query = query.trim();
            nodes.extend(
                pg_query::parse(query)
                    .inspect_err(|_| {
                        eprintln!("Error in {}", file_path.to_string_lossy())
                    })?
                    .protobuf
                    .nodes()
                    .iter()
                    .filter(|a| a.1 == 1)
                    .map(|a| NodeInfo::new(query, a.0.to_enum()))
                    .collect::<Vec<_>>()
            );
        }

        nodes.sort();

        Ok(Self { file_path, nodes })
    }
}

impl TryFrom<&FileMeta> for (Migration, Migration) {
    type Error = Box<dyn Error>;

    fn try_from(value: &FileMeta) -> std::prelude::v1::Result<Self, Self::Error> {
        let file_path = &value.file_path;
        let description = value.description();
        let sql_up = read_to_string(file_path)?;
        let m_up = Migration::new(
            0,
            description.to_string().into(),
            MigrationType::ReversibleUp,
            sql_up.into(),
            false
        );

        let file_path_str = file_path.to_string_lossy().to_string();
        let file_name = &file_path_str[..file_path_str.find('.').unwrap()];
        let app_env = Env::env();

        let mut sql_down = String::default();
        for file in [
            format!("{file_name}.{app_env}.down.sql"),
            format!("{file_name}.down.sql")
        ] {
            let file_path_down = Path::new(&file);
            if file_path_down.exists() {
                sql_down = read_to_string(file_path_down)?;
                break;
            }
        }

        let m_down = Migration::new(
            0,
            description.to_string().into(),
            MigrationType::ReversibleDown,
            sql_down.into(),
            false
        );

        Ok((m_up, m_down))
    }
}

impl Eq for FileMeta {}

impl Ord for FileMeta {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.nodes.is_empty() {
            return Ordering::Greater;
        }

        if other.nodes.is_empty() {
            return Ordering::Less;
        }

        let min_a = self
            .nodes
            .iter()
            .filter(|n| n.exec_priority != NodeInfo::IGNORE_PRIOR)
            .min();

        if min_a.is_none() {
            return Ordering::Greater;
        }

        let min_b = other
            .nodes
            .iter()
            .filter(|n| n.exec_priority != NodeInfo::IGNORE_PRIOR)
            .min();

        if min_b.is_none() {
            return Ordering::Less;
        }

        let order = min_a.unwrap().cmp(min_b.unwrap());

        if order != Ordering::Equal {
            return order;
        }

        let max_a = self
            .nodes
            .iter()
            .filter(|n| n.exec_priority != NodeInfo::IGNORE_PRIOR)
            .max()
            .unwrap();

        let max_b = other
            .nodes
            .iter()
            .filter(|n| n.exec_priority != NodeInfo::IGNORE_PRIOR)
            .max()
            .unwrap();

        max_a.cmp(max_b)
    }
}

impl PartialOrd for FileMeta {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
