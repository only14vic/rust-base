use {
    core::str,
    pg_query::{
        Node, NodeEnum,
        protobuf::{self, ObjectType, RoleStmtType}
    },
    std::{cmp::Ordering, collections::HashSet, fmt::Debug}
};

#[derive(PartialEq)]
pub struct NodeInfo {
    pub exec_priority: i16,
    pub create_target: Option<String>,
    pub drop_targets: Vec<String>,
    pub need_targets: Vec<String>,
    pub ignore_targets: Vec<String>,
    pub resortable: bool,
    pub node: NodeEnum,
    pub query: String
}

impl Debug for NodeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeInfo")
            .field("exec_priority", &self.exec_priority)
            .field("create_target", &self.create_target)
            .field("drop_targets", &self.drop_targets)
            .field("need_targets", &self.need_targets)
            .field("ignore_targets", &self.ignore_targets)
            .field("resortable", &self.resortable)
            .field(
                "query",
                &format!("{}...", &self.query.chars().take(50).collect::<String>())
            )
            .field(
                "node",
                &(format!("{:?}", &self.node)
                    .chars()
                    .take(50)
                    .collect::<String>()
                    + "...")
            )
            .finish()
    }
}

impl Eq for NodeInfo {}

impl Ord for NodeInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.exec_priority.cmp(&other.exec_priority)
    }
}

impl PartialOrd for NodeInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub trait StmtTarget {
    fn target_name(&self) -> Option<String>;
}

impl StmtTarget for Vec<Node> {
    fn target_name(&self) -> Option<String> {
        match self.first() {
            Some(Node { node: Some(NodeEnum::String(_)) }) => {
                let name = self
                    .iter()
                    .filter_map(|e| {
                        match e.node.as_ref() {
                            Some(NodeEnum::String(s)) => Some(s.sval.as_str()),
                            _ => None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(".");

                if name.is_empty() { None } else { Some(name) }
            },
            _ => None
        }
    }
}

impl StmtTarget for protobuf::ObjectWithArgs {
    fn target_name(&self) -> Option<String> {
        self.objname.target_name()
    }
}

impl StmtTarget for protobuf::RangeVar {
    fn target_name(&self) -> Option<String> {
        format!("{}.{}", self.schemaname, self.relname).into()
    }
}

impl StmtTarget for Node {
    fn target_name(&self) -> Option<String> {
        match self.node.as_ref() {
            Some(NodeEnum::String(node)) => node.sval.clone().into(),
            Some(NodeEnum::RoleSpec(role)) => role.rolename.clone().into(),
            Some(NodeEnum::ObjectWithArgs(node)) => node.objname.target_name(),
            Some(NodeEnum::List(list)) => {
                list.items
                    .iter()
                    .filter_map(|a| a.target_name())
                    .collect::<Vec<_>>()
                    .join(".")
                    .into()
            },

            _ => None
        }
    }
}

impl NodeInfo {
    const DROP_PRIOR: i16 = -1;
    const COMMENT_PRIOR: i16 = 5;
    const ALTER_PRIOR: i16 = 6;
    const RENAME_PRIOR: i16 = 7;

    pub const IGNORE_PRIOR: i16 = 30000;
    pub const IGNORE_TARGET_MARK: &str = "@migration-sort-ignore-target:";
    pub const NEED_TARGET_MARK: &str = "@migration-sort-need-target:";

    pub fn new(query: &str, node: NodeEnum) -> Self {
        #[rustfmt::skip]
        let (
            exec_priority,
            create_target,
            drop_targets,
            resortable
        ) = Self::parse_node(&node);

        let ignore_targets = Self::find_ignore_targets_by_marker(query);
        let need_targets = Self::find_need_targets_by_marker(query);

        Self {
            exec_priority,
            create_target,
            drop_targets,
            need_targets,
            ignore_targets,
            resortable,
            node,
            query: query.to_string()
        }
    }

    pub fn create_target(&self) -> Option<&String> {
        self.create_target.as_ref()
    }

    pub fn find_need_targets(&mut self, targets: &[String]) {
        for target_name in targets.iter() {
            if target_name.is_empty() {
                continue;
            }

            if self.ignore_targets.contains(target_name) {
                continue;
            }

            if self.need_targets.contains(target_name) {
                continue;
            }

            if self.create_target.as_ref() == Some(target_name) {
                continue;
            }

            let mut sql = self.query.as_str();
            let mut sql_bytes = sql.as_bytes();
            let mut end_pos = 0;

            loop {
                sql = std::str::from_utf8(&sql_bytes[end_pos..]).unwrap();
                sql_bytes = sql.as_bytes();

                let start_pos = match sql.find(target_name) {
                    Some(pos) => pos,
                    None => break
                };
                end_pos = start_pos + target_name.len();

                let ch_before = std::str::from_utf8(&sql_bytes[..start_pos])
                    .unwrap()
                    .chars()
                    .last();
                let ch_after = std::str::from_utf8(&sql_bytes[end_pos..])
                    .unwrap()
                    .chars()
                    .nth(0);

                let is_match = [ch_before, ch_after].iter().any(|ch| {
                    match ch {
                        Some(ch) => ch.is_alphanumeric() || *ch == '_',
                        None => false
                    }
                }) == false;

                if is_match && ch_before != Some('@') {
                    self.need_targets.push(target_name.to_string());
                    break;
                }
            }
        }
    }

    fn find_targets_by_marker(query: &str, marker: &str) -> Vec<String> {
        query
            .match_indices(marker)
            .filter_map(|(pos, ..)| {
                let target = str::from_utf8(&query.as_bytes()[pos..])
                    .unwrap()
                    .chars()
                    .skip_while(|ch| *ch != ':')
                    .skip(1)
                    .take_while(|ch| ['*', '\n'].contains(ch) == false)
                    .collect::<String>()
                    .split_terminator(&[',', ' '])
                    .filter_map(|s| {
                        if s.is_empty() == false {
                            Some(s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                if target.is_empty() == false { Some(target) } else { None }
            })
            .collect::<Vec<_>>()
            .concat()
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    fn find_ignore_targets_by_marker(query: &str) -> Vec<String> {
        Self::find_targets_by_marker(query, Self::IGNORE_TARGET_MARK)
    }

    fn find_need_targets_by_marker(query: &str) -> Vec<String> {
        Self::find_targets_by_marker(query, Self::NEED_TARGET_MARK)
    }

    fn object_priority(object: ObjectType) -> i16 {
        match object {
            ObjectType::ObjectLanguage => 10,
            ObjectType::ObjectCollation => 20,
            ObjectType::ObjectRole => 100,
            ObjectType::ObjectUserMapping => 200,
            ObjectType::ObjectDatabase => 300,
            ObjectType::ObjectSchema => 400,
            ObjectType::ObjectExtension => 500,
            ObjectType::ObjectStatisticExt => 550,
            ObjectType::ObjectSequence => 600,
            ObjectType::ObjectType => 700,
            ObjectType::ObjectTablespace => 750,
            ObjectType::ObjectTable => 800,
            ObjectType::ObjectView => 900,
            ObjectType::ObjectMatview => 950,
            ObjectType::ObjectForeignServer => 960,
            ObjectType::ObjectFdw => 965,
            ObjectType::ObjectForeignTable => 970,
            ObjectType::ObjectColumn => 980,
            ObjectType::ObjectPublication => 990,
            ObjectType::ObjectPublicationRel => 992,
            ObjectType::ObjectPublicationNamespace => 994,
            ObjectType::ObjectFunction => 1000,
            ObjectType::ObjectProcedure => 1010,
            ObjectType::ObjectAggregate => 1020,
            ObjectType::ObjectRoutine => 1030,
            ObjectType::ObjectAccessMethod => 1200,
            ObjectType::ObjectOperator => 1300,
            ObjectType::ObjectOpclass => 1310,
            ObjectType::ObjectOpfamily => 1320,
            ObjectType::ObjectDomain => 1330,
            ObjectType::ObjectCast => 1340,
            ObjectType::ObjectTransform => 1350,
            ObjectType::ObjectConversion => 1360,
            ObjectType::ObjectIndex => 1400,
            ObjectType::ObjectTrigger => 1500,
            ObjectType::ObjectEventTrigger => 1550,
            ObjectType::ObjectPolicy => 1600,
            ObjectType::ObjectRule => 1650,
            ObjectType::ObjectSubscription => 1700,
            ObjectType::Undefined => Self::IGNORE_PRIOR,
            _ => Self::IGNORE_PRIOR
        }
    }

    fn parse_node(node: &NodeEnum) -> (i16, Option<String>, Vec<String>, bool) {
        match node {
            NodeEnum::DoStmt(_) => {
                (
                    Self::IGNORE_PRIOR + 1000,
                    Default::default(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::CommentStmt(stmt) => {
                (
                    Self::object_priority(stmt.objtype()) + Self::COMMENT_PRIOR,
                    Default::default(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::RenameStmt(stmt) => {
                let oldname = stmt
                    .object
                    .as_ref()
                    .map(|a| a.target_name())
                    .unwrap_or_else(|| {
                        stmt.relation
                            .as_ref()
                            .map(|a| a.target_name())
                            .unwrap_or_default()
                    })
                    .expect("Empty rename object name");

                (
                    Self::object_priority(stmt.rename_type()) + Self::RENAME_PRIOR,
                    format!(
                        "{}{dot}{}",
                        &oldname.split_once('.').map(|n| n.0).unwrap_or_default(),
                        &stmt.newname,
                        dot = if oldname.contains(".") { "." } else { "" }
                    )
                    .into(),
                    [oldname.clone()].into(),
                    true
                )
            },
            NodeEnum::DropStmt(stmt) => {
                (
                    Self::object_priority(stmt.remove_type()) + Self::DROP_PRIOR,
                    Default::default(),
                    stmt.objects
                        .iter()
                        .filter_map(|o| o.target_name())
                        .collect(),
                    match stmt.remove_type() {
                        ObjectType::ObjectTable
                        | ObjectType::ObjectView
                        | ObjectType::ObjectMatview
                        | ObjectType::ObjectFunction
                        | ObjectType::ObjectRole => true,
                        _ => Default::default()
                    }
                )
            },
            NodeEnum::DropSubscriptionStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectSubscription)
                        + Self::DROP_PRIOR,
                    Default::default(),
                    [stmt.subname.clone()].into(),
                    Default::default()
                )
            },

            NodeEnum::CreateSubscriptionStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectSubscription),
                    stmt.subname.clone().into(),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::AlterSubscriptionStmt(_) => {
                (
                    Self::object_priority(ObjectType::ObjectSubscription)
                        + Self::ALTER_PRIOR,
                    Default::default(),
                    Default::default(),
                    Default::default()
                )
            },

            NodeEnum::DropRoleStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectRole) + Self::DROP_PRIOR,
                    Default::default(),
                    stmt.roles.iter().filter_map(|r| r.target_name()).collect(),
                    true
                )
            },
            NodeEnum::CreateRoleStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectRole)
                        + match stmt.stmt_type() {
                            RoleStmtType::RolestmtRole => 0,
                            RoleStmtType::RolestmtGroup => 1,
                            RoleStmtType::RolestmtUser => 2,
                            RoleStmtType::Undefined => 3
                        },
                    stmt.role.clone().into(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::AlterRoleStmt(_) | NodeEnum::AlterRoleSetStmt(_) => {
                (
                    Self::object_priority(ObjectType::ObjectRole)
                        + 10
                        + Self::ALTER_PRIOR,
                    Default::default(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::DropUserMappingStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectUserMapping)
                        + Self::DROP_PRIOR,
                    Default::default(),
                    stmt.user
                        .as_ref()
                        .map(|u| [u.rolename.clone()].into())
                        .unwrap_or_default(),
                    Default::default()
                )
            },
            NodeEnum::CreateUserMappingStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectUserMapping),
                    stmt.user.as_ref().map(|a| a.rolename.clone()),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::DropdbStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectDatabase) + Self::DROP_PRIOR,
                    Default::default(),
                    [stmt.dbname.clone()].into(),
                    Default::default()
                )
            },
            NodeEnum::CreatedbStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectDatabase),
                    stmt.dbname.clone().into(),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::AlterDatabaseStmt(_) => {
                (
                    Self::object_priority(ObjectType::ObjectDatabase) + Self::ALTER_PRIOR,
                    Default::default(),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::CreateSchemaStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectSchema),
                    stmt.schemaname.clone().into(),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::AlterObjectSchemaStmt(stmt) => {
                (
                    Self::object_priority(stmt.object_type()) + Self::ALTER_PRIOR,
                    format!(
                        "{}.{}",
                        &stmt.newschema,
                        &stmt
                            .object
                            .as_ref()
                            .map(|a| a.target_name())
                            .unwrap_or_else(|| {
                                stmt.relation
                                    .as_ref()
                                    .map(|a| a.target_name())
                                    .unwrap_or_default()
                            })
                            .expect("Empty object name")
                            .split_once('.')
                            .map(|n| n.1)
                            .unwrap_or_default()
                    )
                    .into(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::CreateExtensionStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectExtension),
                    stmt.extname.clone().into(),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::AlterExtensionStmt(_) => {
                (
                    Self::object_priority(ObjectType::ObjectExtension)
                        + Self::ALTER_PRIOR,
                    Default::default(),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::CreateSeqStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectSequence),
                    stmt.sequence
                        .as_ref()
                        .map(|a| a.target_name())
                        .expect("Empty seq name"),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::AlterSeqStmt(_) => {
                (
                    Self::object_priority(ObjectType::ObjectSequence) + Self::ALTER_PRIOR,
                    Default::default(),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::CreateEnumStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectType),
                    stmt.type_name
                        .target_name()
                        .expect("Empty enum name")
                        .into(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::AlterEnumStmt(_) => {
                (
                    Self::object_priority(ObjectType::ObjectType)
                        + Self::ALTER_PRIOR
                        + 50,
                    Default::default(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::CreateRangeStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectType) + 10,
                    stmt.type_name.target_name(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::CompositeTypeStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectType) + 20,
                    stmt.typevar
                        .as_ref()
                        .map(|a| a.target_name())
                        .expect("Empty type name"),
                    Default::default(),
                    true
                )
            },
            NodeEnum::AlterTypeStmt(_) => {
                (
                    Self::object_priority(ObjectType::ObjectType)
                        + Self::ALTER_PRIOR
                        + 50,
                    Default::default(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::CreateStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectTable),
                    stmt.relation
                        .as_ref()
                        .map(|a| a.target_name())
                        .expect("Empty create stmt"),
                    Default::default(),
                    true
                )
            },
            NodeEnum::AlterTableStmt(stmt) => {
                (
                    Self::object_priority(stmt.objtype()) + Self::ALTER_PRIOR,
                    Default::default(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::ViewStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectView),
                    stmt.view
                        .as_ref()
                        .map(|a| a.target_name())
                        .expect("Empty view name"),
                    Default::default(),
                    true
                )
            },
            NodeEnum::CreateTableAsStmt(stmt) => {
                (
                    Self::object_priority(stmt.objtype()),
                    stmt.into
                        .as_ref()
                        .map(|a| {
                            a.rel
                                .as_ref()
                                .map(|r| r.target_name())
                                .expect("Empty table name")
                        })
                        .expect("Empty table name"),
                    Default::default(),
                    true
                )
            },
            NodeEnum::DefineStmt(stmt) => {
                (
                    Self::object_priority(stmt.kind()),
                    stmt.defnames.target_name(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::CreateFunctionStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectFunction),
                    stmt.funcname.target_name(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::AlterFunctionStmt(_) => {
                (
                    Self::object_priority(ObjectType::ObjectFunction) + Self::ALTER_PRIOR,
                    Default::default(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::CreateAmStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectAccessMethod),
                    stmt.amname.clone().into(),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::CreateOpClassStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectOpclass),
                    stmt.opclassname.target_name(),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::CreateOpClassItem(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectOpclass) + 2,
                    stmt.name
                        .as_ref()
                        .map(|a| a.target_name())
                        .expect("Empty opclassitem name"),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::CreateOpFamilyStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectOpfamily),
                    stmt.opfamilyname.target_name(),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::CreateDomainStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectDomain),
                    stmt.domainname.target_name(),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::CreateCastStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectCast),
                    stmt.sourcetype
                        .as_ref()
                        .map(|a| a.names.target_name())
                        .expect("Empty cast name"),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::CreateTransformStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectTransform),
                    stmt.type_name
                        .as_ref()
                        .map(|a| a.names.target_name())
                        .expect("Empty transform name"),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::CreateConversionStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectConversion),
                    stmt.conversion_name.target_name(),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::IndexStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectIndex),
                    stmt.idxname.clone().into(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::ReindexStmt(_) => {
                (
                    Self::object_priority(ObjectType::ObjectIndex) + 2,
                    Default::default(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::CreateTrigStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectTrigger),
                    stmt.trigname.clone().into(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::CreateEventTrigStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectEventTrigger),
                    stmt.trigname.clone().into(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::CreatePolicyStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectPolicy),
                    stmt.policy_name.clone().into(),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::AlterPolicyStmt(_) => {
                (
                    Self::object_priority(ObjectType::ObjectPolicy) + Self::ALTER_PRIOR,
                    Default::default(),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::RuleStmt(stmt) => {
                (
                    Self::object_priority(ObjectType::ObjectRule),
                    stmt.rulename.clone().into(),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::GrantRoleStmt(stmt) => {
                (
                    if stmt.is_grant { 1900 } else { 1900 - 1 },
                    Default::default(),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::GrantStmt(stmt) => {
                (
                    if stmt.is_grant { 1910 } else { 1910 - 1 },
                    Default::default(),
                    Default::default(),
                    Default::default()
                )
            },
            NodeEnum::DeleteStmt(_) => {
                (
                    Self::IGNORE_PRIOR + 10,
                    Default::default(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::InsertStmt(_) => {
                (
                    Self::IGNORE_PRIOR + 11,
                    Default::default(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::UpdateStmt(_) => {
                (
                    Self::IGNORE_PRIOR + 12,
                    Default::default(),
                    Default::default(),
                    true
                )
            },
            NodeEnum::SelectStmt(_) => {
                (
                    Self::IGNORE_PRIOR + 13,
                    Default::default(),
                    Default::default(),
                    true
                )
            },
            _ => {
                (
                    Self::object_priority(ObjectType::Undefined),
                    Default::default(),
                    Default::default(),
                    true
                )
            },
        }
    }
}
