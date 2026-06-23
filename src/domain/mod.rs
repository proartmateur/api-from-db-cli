use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseEngine {
    PostgreSql,
    SqlServer,
}

impl fmt::Display for DatabaseEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PostgreSql => write!(f, "postgresql"),
            Self::SqlServer => write!(f, "sqlserver"),
        }
    }
}

impl DatabaseEngine {
    pub fn default_port(self) -> u16 {
        match self {
            Self::PostgreSql => 5432,
            Self::SqlServer => 1433,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseObjectType {
    Table,
    Function,
    StoredProcedure,
}

impl DatabaseObjectType {
    fn sort_priority(self) -> u8 {
        match self {
            Self::Table => 0,
            Self::Function => 1,
            Self::StoredProcedure => 2,
        }
    }
}

pub fn sort_objects(objects: &mut Vec<DatabaseObject>) {
    objects.sort_by(|a, b| {
        a.object_type
            .sort_priority()
            .cmp(&b.object_type.sort_priority())
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub engine: DatabaseEngine,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub connection_string: Option<String>,
    pub config_file_path: Option<String>,
    pub generator: GeneratorConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorConfig {
    pub cmd: String,
    pub flags: Vec<String>,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            cmd: "gen.exe".to_string(),
            flags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseObject {
    pub name: String,
    pub schema: Option<String>,
    pub object_type: DatabaseObjectType,
    pub engine: DatabaseEngine,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableSchema {
    pub schema: String,
    pub name: String,
    pub columns: Vec<ColumnSchema>,
    pub primary_keys: Vec<String>,
}

impl TableSchema {
    pub fn new(
        schema: impl Into<String>,
        name: impl Into<String>,
        columns: Vec<ColumnSchema>,
        primary_keys: Vec<String>,
    ) -> Self {
        Self {
            schema: schema.into(),
            name: name.into(),
            columns,
            primary_keys,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnSchema {
    pub name: String,
    pub db_type: String,
    pub normalized_type: NormalizedType,
    pub nullable: bool,
    pub ordinal_position: usize,
    pub is_primary_key: bool,
    pub is_deleted_at_candidate: bool,
}

impl ColumnSchema {
    pub fn new(
        name: impl Into<String>,
        db_type: impl Into<String>,
        nullable: bool,
        ordinal_position: usize,
    ) -> Self {
        Self {
            name: name.into(),
            db_type: db_type.into(),
            normalized_type: NormalizedType::Unknown,
            nullable,
            ordinal_position,
            is_primary_key: false,
            is_deleted_at_candidate: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizedType {
    Int,
    Str,
    Bool,
    Float,
    Decimal,
    Datetime,
    Date,
    Time,
    Dict,
    DeleteAt,
    Unknown,
}

impl NormalizedType {
    pub fn as_generator_token(&self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Str => "str",
            Self::Bool => "bool",
            Self::Float => "float",
            Self::Decimal => "Decimal",
            Self::Datetime => "datetime",
            Self::Date => "date",
            Self::Time => "time",
            Self::Dict => "dict",
            Self::DeleteAt => "delete_at",
            Self::Unknown => "str",
        }
    }
}

impl fmt::Display for NormalizedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_generator_token())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftDeleteConfig {
    pub enabled: bool,
    pub field: Option<String>,
    pub field_was_detected: bool,
    pub field_was_selected_manually: bool,
    pub field_was_created: bool,
}

impl SoftDeleteConfig {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            field: None,
            field_was_detected: false,
            field_was_selected_manually: false,
            field_was_created: false,
        }
    }

    pub fn summary(&self) -> String {
        match (self.enabled, self.field.as_deref()) {
            (false, _) => "deshabilitado".to_string(),
            (true, Some(field)) if self.field_was_detected => {
                format!("detectado automaticamente en `{field}`")
            }
            (true, Some(field)) if self.field_was_selected_manually => {
                format!("seleccionado manualmente en `{field}`")
            }
            (true, Some(field)) if self.field_was_created => {
                format!("requiere crear `{field}`")
            }
            (true, Some(field)) => format!("configurado en `{field}`"),
            (true, None) => "pendiente de definir".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedSql {
    pub engine: DatabaseEngine,
    pub schema: String,
    pub table: String,
    pub sql: String,
    pub operation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedCommand {
    pub executable: String,
    pub table_name: String,
    pub arguments: Vec<String>,
    pub raw_command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoftDeletePreference {
    PreferDeleteEndpoint,
    SkipDeleteEndpoint,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(name: &str, object_type: DatabaseObjectType) -> DatabaseObject {
        DatabaseObject {
            name: name.to_string(),
            schema: None,
            object_type,
            engine: DatabaseEngine::PostgreSql,
        }
    }

    #[test]
    fn sort_objects_tables_first_then_alphabetical() {
        let mut objects = vec![
            obj("zebra_func", DatabaseObjectType::Function),
            obj("b_table", DatabaseObjectType::Table),
            obj("a_proc", DatabaseObjectType::StoredProcedure),
            obj("a_table", DatabaseObjectType::Table),
        ];
        sort_objects(&mut objects);
        assert_eq!(objects[0].name, "a_table");
        assert_eq!(objects[1].name, "b_table");
        assert_eq!(objects[2].name, "zebra_func");
        assert_eq!(objects[3].name, "a_proc");
    }
}