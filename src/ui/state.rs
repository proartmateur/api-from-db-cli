use std::path::PathBuf;

use crate::app::generation_service::GenerationPreview;
use crate::domain::{ConnectionConfig, DatabaseEngine, DatabaseObject, ProcessResult, TableSchema};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Screen {
    ConnectionSource,
    EngineSelect,
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
pub(crate) struct ObjectExplorerScreenState {
    pub(crate) selected_object: usize,
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
