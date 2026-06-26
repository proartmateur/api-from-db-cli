use std::path::PathBuf;

use crate::app::generation_service::GenerationPreview;
use crate::domain::{ConnectionConfig, DatabaseEngine, DatabaseObject, ProcessResult, TableSchema};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Screen {
    ConnectionSource,
    EngineSelect,
    ManualConnection,
    ObjectExplorer,
    ObjectDetails,
    SoftDeleteStrategy,
    ManualSoftDeleteField,
    SqlPreview,
    CommandPreview,
    ProcessResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConnectionSource {
    Manual,
    ConfigFile,
}

impl ConnectionSource {
    pub(crate) const ALL: [Self; 2] = [Self::Manual, Self::ConfigFile];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Manual => "Captura manual",
            Self::ConfigFile => "Archivo de configuracion",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SoftDeleteStrategy {
    CreateDeletedAt,
    UseExistingField,
    ContinueWithoutDelete,
}

impl SoftDeleteStrategy {
    pub(crate) const ALL: [Self; 3] = [
        Self::CreateDeletedAt,
        Self::UseExistingField,
        Self::ContinueWithoutDelete,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::CreateDeletedAt => "Crear campo deleted_at",
            Self::UseExistingField => "Usar columna existente",
            Self::ContinueWithoutDelete => "Continuar sin endpoint delete",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EngineOption {
    PostgreSql,
    SqlServer,
}

impl EngineOption {
    pub(crate) const ALL: [Self; 2] = [Self::PostgreSql, Self::SqlServer];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::PostgreSql => "PostgreSQL",
            Self::SqlServer => "SQL Server",
        }
    }

    pub(crate) fn to_engine(self) -> DatabaseEngine {
        match self {
            Self::PostgreSql => DatabaseEngine::PostgreSql,
            Self::SqlServer => DatabaseEngine::SqlServer,
        }
    }

    pub(crate) fn default_port(self) -> u16 {
        self.to_engine().default_port()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ManualConnectionField {
    Host,
    Port,
    Database,
    Username,
    Password,
    Connect,
}

impl ManualConnectionField {
    pub(crate) const ALL: [Self; 6] = [
        Self::Host,
        Self::Port,
        Self::Database,
        Self::Username,
        Self::Password,
        Self::Connect,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Host => "Host",
            Self::Port => "Port",
            Self::Database => "Database",
            Self::Username => "Username",
            Self::Password => "Password",
            Self::Connect => "Conectar",
        }
    }

    pub(crate) fn is_editable(self) -> bool {
        !matches!(self, Self::Connect)
    }

    pub(crate) fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|f| *f == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub(crate) fn previous(self) -> Self {
        let idx = Self::ALL.iter().position(|f| *f == self).unwrap_or(0);
        Self::ALL[(idx + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SqlPreviewAction {
    ExecuteAndContinue,
    CopyAndContinue,
    CopyAndStay,
    Cancel,
}

impl SqlPreviewAction {
    pub(crate) const ALL: [Self; 4] = [
        Self::ExecuteAndContinue,
        Self::CopyAndContinue,
        Self::CopyAndStay,
        Self::Cancel,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::ExecuteAndContinue => "Ejecutar ALTER TABLE y continuar",
            Self::CopyAndContinue => "Copiar SQL y continuar al comando",
            Self::CopyAndStay => "Copiar SQL y quedarse aqui",
            Self::Cancel => "Cancelar",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommandPreviewAction {
    ExecuteCommand,
    CopyAndMarkExternalExecution,
    CopyAndStay,
    Cancel,
}

impl CommandPreviewAction {
    pub(crate) const ALL: [Self; 4] = [
        Self::ExecuteCommand,
        Self::CopyAndMarkExternalExecution,
        Self::CopyAndStay,
        Self::Cancel,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::ExecuteCommand => "Ejecutar comando",
            Self::CopyAndMarkExternalExecution => "Copiar comando y marcar ejecucion externa",
            Self::CopyAndStay => "Copiar comando y quedarse aqui",
            Self::Cancel => "Cancelar",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CatalogMode {
    Mock,
    Real,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GeneratorBinaryState {
    Available { path: PathBuf },
    Missing { searched_paths: Vec<PathBuf> },
    Error(String),
}

#[derive(Debug, Clone)]
pub(crate) struct MockTable {
    pub(crate) schema: TableSchema,
}

#[derive(Debug, Clone)]
pub(crate) struct Catalog {
    pub(crate) objects: Vec<DatabaseObject>,
    pub(crate) tables: Vec<MockTable>,
    pub(crate) mode: CatalogMode,
}

#[derive(Debug, Clone)]
pub(crate) struct ConnectionSourceScreenState {
    pub(crate) selected_source: ConnectionSource,
}

#[derive(Debug, Clone)]
pub(crate) struct EngineSelectScreenState {
    pub(crate) selected_engine: EngineOption,
}

#[derive(Debug, Clone)]
pub(crate) struct ManualConnectionScreenState {
    pub(crate) selected_field: ManualConnectionField,
    pub(crate) editing: bool,
    pub(crate) host: String,
    pub(crate) port: String,
    pub(crate) database: String,
    pub(crate) username: String,
    pub(crate) password: String,
}

impl ManualConnectionScreenState {
    pub(crate) fn for_engine(engine: EngineOption) -> Self {
        Self {
            selected_field: ManualConnectionField::Host,
            editing: false,
            host: "localhost".to_string(),
            port: engine.default_port().to_string(),
            database: String::new(),
            username: String::new(),
            password: String::new(),
        }
    }

    pub(crate) fn value(&self, field: ManualConnectionField) -> &str {
        match field {
            ManualConnectionField::Host => &self.host,
            ManualConnectionField::Port => &self.port,
            ManualConnectionField::Database => &self.database,
            ManualConnectionField::Username => &self.username,
            ManualConnectionField::Password => &self.password,
            ManualConnectionField::Connect => "",
        }
    }

    pub(crate) fn value_mut(&mut self, field: ManualConnectionField) -> &mut String {
        match field {
            ManualConnectionField::Host => &mut self.host,
            ManualConnectionField::Port => &mut self.port,
            ManualConnectionField::Database => &mut self.database,
            ManualConnectionField::Username => &mut self.username,
            ManualConnectionField::Password => &mut self.password,
            ManualConnectionField::Connect => {
                panic!("Connect no es un campo editable");
            }
        }
    }

    pub(crate) fn reset_for_engine(&mut self, engine: EngineOption) {
        *self = Self::for_engine(engine);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ObjectExplorerScreenState {
    pub(crate) selected_object: usize,
    pub(crate) searching: bool,
    pub(crate) filter: String,
}

#[derive(Debug, Clone)]
pub(crate) struct SoftDeleteScreenState {
    pub(crate) selected_strategy: SoftDeleteStrategy,
    pub(crate) selected_manual_field: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct SqlPreviewScreenState {
    pub(crate) selected_action: SqlPreviewAction,
}

#[derive(Debug, Clone)]
pub(crate) struct CommandPreviewScreenState {
    pub(crate) selected_action: CommandPreviewAction,
}

#[derive(Debug, Clone)]
pub(crate) struct ProcessResultScreenState {
    pub(crate) scroll: u16,
}

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub(crate) screen: Screen,
    pub(crate) connection_source_screen: ConnectionSourceScreenState,
    pub(crate) engine_select_screen: EngineSelectScreenState,
    pub(crate) manual_connection_screen: ManualConnectionScreenState,
    pub(crate) object_explorer_screen: ObjectExplorerScreenState,
    pub(crate) soft_delete_screen: SoftDeleteScreenState,
    pub(crate) sql_preview_screen: SqlPreviewScreenState,
    pub(crate) command_preview_screen: CommandPreviewScreenState,
    pub(crate) process_result_screen: ProcessResultScreenState,
    pub(crate) connection_source: Option<ConnectionSource>,
    pub(crate) engine: Option<DatabaseEngine>,
    pub(crate) catalog: Catalog,
    pub(crate) selected_db_object: Option<DatabaseObject>,
    pub(crate) preview: Option<GenerationPreview>,
    pub(crate) process_result: Option<ProcessResult>,
    pub(crate) config_file_path: Option<String>,
    pub(crate) connection_config: Option<ConnectionConfig>,
    pub(crate) generator_binary_state: GeneratorBinaryState,
    pub(crate) last_message: String,
}
