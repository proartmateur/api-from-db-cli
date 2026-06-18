use std::env;
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal};

use crate::adapters::config::ConfigLoader;
use crate::adapters::process::StdProcessRunner;
use crate::adapters::sqlserver::SqlServerAdapter;
use crate::app::generation_service::{GenerationPreview, GenerationService};
use crate::core::ports::{ConnectionProvider, MetadataExplorer, ProcessRunner};
use crate::domain::{
    ColumnSchema, ConnectionConfig, DatabaseEngine, DatabaseObject, DatabaseObjectType,
    ProcessResult, SoftDeletePreference, TableSchema,
};
use crate::ui::screens;

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

    fn to_engine(self) -> DatabaseEngine {
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

pub struct TuiApp {
    pub(crate) state: AppState,
}

impl TuiApp {
    pub fn new() -> Self {
        Self {
            state: AppState {
                screen: Screen::ConnectionSource,
                connection_source_screen: ConnectionSourceScreenState {
                    selected_source: ConnectionSource::Manual,
                },
                engine_select_screen: EngineSelectScreenState {
                    selected_engine: EngineOption::PostgreSql,
                },
                object_explorer_screen: ObjectExplorerScreenState { selected_object: 0 },
                soft_delete_screen: SoftDeleteScreenState {
                    selected_strategy: SoftDeleteStrategy::CreateDeletedAt,
                    selected_manual_field: 0,
                },
                sql_preview_screen: SqlPreviewScreenState {
                    selected_action: SqlPreviewAction::ExecuteAndContinue,
                },
                command_preview_screen: CommandPreviewScreenState {
                    selected_action: CommandPreviewAction::ExecuteCommand,
                },
                process_result_screen: ProcessResultScreenState { scroll: 0 },
                connection_source: None,
                engine: None,
                catalog: mock_catalog(DatabaseEngine::PostgreSql),
                selected_db_object: None,
                preview: None,
                process_result: None,
                config_file_path: None,
                connection_config: None,
                generator_binary_state: detect_generator_binary_state(),
                last_message: initial_connection_message(),
            },
        }
    }

    pub fn run(mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.event_loop(&mut terminal);
        let restore_result = restore_terminal(terminal);

        result.and(restore_result)
    }

    fn event_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(Duration::from_millis(200))? {
                let Event::Key(key) = event::read()? else {
                    continue;
                };

                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if matches!(key.code, KeyCode::Char('q')) {
                    return Ok(());
                }

                self.handle_key(key.code);
            }
        }
    }

    fn handle_key(&mut self, code: KeyCode) {
        match self.state.screen {
            Screen::ConnectionSource => self.handle_connection_source(code),
            Screen::EngineSelect => self.handle_engine_select(code),
            Screen::ObjectExplorer => self.handle_object_explorer(code),
            Screen::ObjectDetails => self.handle_object_details(code),
            Screen::SoftDeleteStrategy => self.handle_soft_delete_strategy(code),
            Screen::ManualSoftDeleteField => self.handle_manual_soft_delete_field(code),
            Screen::SqlPreview => self.handle_sql_preview(code),
            Screen::CommandPreview => self.handle_command_preview(code),
            Screen::ProcessResult => self.handle_process_result(code),
        }
    }

    fn refresh_generator_binary_state(&mut self) {
        self.state.generator_binary_state = detect_generator_binary_state();
    }

    fn ensure_generator_binary_ready(&mut self) -> bool {
        self.refresh_generator_binary_state();

        match &self.state.generator_binary_state {
            GeneratorBinaryState::Available { path } => {
                self.state.last_message = format!(
                    "Generador detectado en `{}`. Ya puedes continuar.",
                    path.display()
                );
                true
            }
            GeneratorBinaryState::Missing { searched_paths } => {
                let searched = searched_paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(" o ");
                self.state.last_message = format!(
                    "No se encontro `gen` ni `gen.exe` en la raiz del proyecto. Colocalo en {} y presiona Enter para reintentar.",
                    searched
                );
                false
            }
            GeneratorBinaryState::Error(error) => {
                self.state.last_message = format!(
                    "No fue posible validar el generador en la raiz del proyecto: {}",
                    error
                );
                false
            }
        }
    }

    fn handle_config_file_selection(&mut self) {
        let loader = ConfigLoader;
        self.state.connection_source = Some(ConnectionSource::ConfigFile);

        match loader.ensure_default_file(None) {
            Ok(result) if result.created => {
                let path = result.path.display().to_string();
                self.state.config_file_path = Some(path.clone());
                self.state.last_message = format!(
                    "Se genero `{}`. Editalo manualmente y presiona Enter otra vez para cargarlo.",
                    path
                );
            }
            Ok(result) => {
                let path = result.path.display().to_string();
                self.state.config_file_path = Some(path.clone());

                match loader.load_from_json_file(result.path.as_path()) {
                    Ok(config) => self.activate_config_file_connection(config, path),
                    Err(error) => {
                        self.state.last_message = format!(
                            "No se pudo cargar `{}`: {}. Edita el archivo y presiona Enter otra vez.",
                            path, error
                        );
                    }
                }
            }
            Err(error) => {
                self.state.last_message = format!(
                    "No fue posible preparar el archivo de configuracion: {}",
                    error
                );
            }
        }
    }

    fn activate_config_file_connection(&mut self, config: ConnectionConfig, path: String) {
        self.state.connection_config = Some(config.clone());
        self.state.engine = Some(config.engine);
        self.state.object_explorer_screen.selected_object = 0;
        self.state.preview = None;
        self.state.process_result = None;
        self.state.selected_db_object = None;

        match config.engine {
            DatabaseEngine::SqlServer => {
                let adapter = SqlServerAdapter;
                match adapter.test_connection(&config) {
                    Ok(()) => match adapter.list_objects(&config) {
                        Ok(objects) => {
                            self.state.catalog = Catalog {
                                objects,
                                tables: Vec::new(),
                                mode: CatalogMode::Real,
                            };
                            self.state.screen = Screen::ObjectExplorer;
                            self.state.last_message = format!(
                                "Configuracion cargada desde `{}`. Conexion real a SQL Server establecida.",
                                path
                            );
                        }
                        Err(error) => {
                            self.state.last_message = format!(
                                "La conexion a SQL Server funciono, pero no se pudieron listar objetos: {}",
                                error
                            );
                        }
                    },
                    Err(error) => {
                        self.state.last_message =
                            format!("No se pudo conectar a SQL Server con `{}`: {}", path, error);
                    }
                }
            }
            DatabaseEngine::PostgreSql => {
                self.state.catalog = mock_catalog(DatabaseEngine::PostgreSql);
                self.state.screen = Screen::ObjectExplorer;
                self.state.last_message = format!(
                    "Configuracion cargada desde `{}`. PostgreSQL aun usa catalogo mock mientras conectamos su adapter real.",
                    path
                );
            }
        }
    }

    fn handle_connection_source(&mut self, code: KeyCode) {
        match code {
            KeyCode::Up => {
                self.state.connection_source_screen.selected_source =
                    previous_connection_source(self.state.connection_source_screen.selected_source);
            }
            KeyCode::Down => {
                self.state.connection_source_screen.selected_source =
                    next_connection_source(self.state.connection_source_screen.selected_source);
            }
            KeyCode::Enter => {
                if !self.ensure_generator_binary_ready() {
                    return;
                }

                match self.state.connection_source_screen.selected_source {
                    ConnectionSource::Manual => {
                        self.state.connection_source = Some(ConnectionSource::Manual);
                        self.state.connection_config = None;
                        self.state.catalog = mock_catalog(DatabaseEngine::PostgreSql);
                        self.state.screen = Screen::EngineSelect;
                        self.state.last_message =
                            "Generador validado. Conexion mock preparada. Ahora elige el motor de base de datos."
                                .to_string();
                    }
                    ConnectionSource::ConfigFile => self.handle_config_file_selection(),
                }
            }
            _ => {}
        }
    }

    fn handle_engine_select(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.state.screen = Screen::ConnectionSource;
                self.state.last_message =
                    "Regresaste a la seleccion del origen de conexion.".to_string();
            }
            KeyCode::Up => {
                self.state.engine_select_screen.selected_engine =
                    previous_engine_option(self.state.engine_select_screen.selected_engine);
            }
            KeyCode::Down => {
                self.state.engine_select_screen.selected_engine =
                    next_engine_option(self.state.engine_select_screen.selected_engine);
            }
            KeyCode::Enter => {
                let engine = self.state.engine_select_screen.selected_engine.to_engine();
                self.state.engine = Some(engine);
                self.state.catalog = mock_catalog(engine);
                self.state.connection_config = None;
                self.state.object_explorer_screen.selected_object = 0;
                self.state.preview = None;
                self.state.process_result = None;
                self.state.screen = Screen::ObjectExplorer;
                self.state.last_message = format!(
                    "Conexion exitosa simulada a {}. Explora los objetos disponibles.",
                    engine
                );
            }
            _ => {}
        }
    }

    fn handle_object_explorer(&mut self, code: KeyCode) {
        let total = self.state.catalog.objects.len();
        match code {
            KeyCode::Esc => {
                self.state.screen =
                    if self.state.connection_source == Some(ConnectionSource::ConfigFile) {
                        Screen::ConnectionSource
                    } else {
                        Screen::EngineSelect
                    };
                self.state.last_message =
                    "Puedes cambiar de origen o motor sin perder control del flujo.".to_string();
            }
            KeyCode::Up => select_previous(
                &mut self.state.object_explorer_screen.selected_object,
                total,
            ),
            KeyCode::Down => select_next(
                &mut self.state.object_explorer_screen.selected_object,
                total,
            ),
            KeyCode::Enter => {
                if let Some(object) = self
                    .state
                    .catalog
                    .objects
                    .get(self.state.object_explorer_screen.selected_object)
                    .cloned()
                {
                    self.state.selected_db_object = Some(object.clone());
                    self.state.screen = Screen::ObjectDetails;
                    self.state.last_message = format!(
                        "Inspeccionando {} `{}`.",
                        object_type_label(object.object_type),
                        object.name
                    );

                    if object.object_type == DatabaseObjectType::Table {
                        self.state.preview = self.load_preview_for_selected_table(
                            SoftDeletePreference::PreferDeleteEndpoint,
                            None,
                        );
                    } else {
                        self.state.preview = None;
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_object_details(&mut self, code: KeyCode) {
        let Some(selected) = self.state.selected_db_object.clone() else {
            self.state.screen = Screen::ObjectExplorer;
            return;
        };

        match code {
            KeyCode::Esc => {
                self.state.screen = Screen::ObjectExplorer;
                self.state.last_message = "Regresaste al explorador de objetos.".to_string();
            }
            KeyCode::Enter if selected.object_type == DatabaseObjectType::Table => {
                if self.state.preview.is_none() {
                    self.state.last_message =
                        "No fue posible preparar la tabla. Revisa la conexion o el schema."
                            .to_string();
                } else if self.needs_soft_delete_decision() {
                    self.state.screen = Screen::SoftDeleteStrategy;
                    self.state.soft_delete_screen.selected_strategy =
                        SoftDeleteStrategy::CreateDeletedAt;
                    self.state.last_message =
                        "No existe deleted_at compatible. Decide como quieres resolver soft delete."
                            .to_string();
                } else {
                    self.state.screen = Screen::CommandPreview;
                    self.state.command_preview_screen.selected_action =
                        CommandPreviewAction::ExecuteCommand;
                    self.state.last_message =
                        "La tabla ya tiene suficiente metadata para construir el comando."
                            .to_string();
                }
            }
            _ => {}
        }
    }

    fn handle_soft_delete_strategy(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.state.screen = Screen::ObjectDetails;
                self.state.last_message =
                    "Puedes revisar de nuevo la tabla antes de decidir.".to_string();
            }
            KeyCode::Up => {
                self.state.soft_delete_screen.selected_strategy =
                    previous_soft_delete_strategy(self.state.soft_delete_screen.selected_strategy);
            }
            KeyCode::Down => {
                self.state.soft_delete_screen.selected_strategy =
                    next_soft_delete_strategy(self.state.soft_delete_screen.selected_strategy);
            }
            KeyCode::Enter => match self.state.soft_delete_screen.selected_strategy {
                SoftDeleteStrategy::CreateDeletedAt => {
                    self.state.preview = self.load_preview_for_selected_table(
                        SoftDeletePreference::PreferDeleteEndpoint,
                        None,
                    );
                    self.state.screen = Screen::SqlPreview;
                    self.state.sql_preview_screen.selected_action =
                        SqlPreviewAction::ExecuteAndContinue;
                    self.state.last_message =
                        "Se genero el ALTER TABLE para revisar antes de continuar.".to_string();
                }
                SoftDeleteStrategy::UseExistingField => {
                    self.state.soft_delete_screen.selected_manual_field = 0;
                    self.state.screen = Screen::ManualSoftDeleteField;
                    self.state.last_message =
                        "Selecciona una columna datetime existente para soft delete.".to_string();
                }
                SoftDeleteStrategy::ContinueWithoutDelete => {
                    self.state.preview = self.load_preview_for_selected_table(
                        SoftDeletePreference::SkipDeleteEndpoint,
                        None,
                    );
                    self.state.screen = Screen::CommandPreview;
                    self.state.command_preview_screen.selected_action =
                        CommandPreviewAction::ExecuteCommand;
                    self.state.last_message =
                        "Seguimos sin endpoint delete para esta API.".to_string();
                }
            },
            _ => {}
        }
    }

    fn handle_manual_soft_delete_field(&mut self, code: KeyCode) {
        let total = self.manual_soft_delete_candidates().len();
        match code {
            KeyCode::Esc => {
                self.state.screen = Screen::SoftDeleteStrategy;
                self.state.last_message =
                    "Regresaste a las opciones para resolver soft delete.".to_string();
            }
            KeyCode::Up => select_previous(
                &mut self.state.soft_delete_screen.selected_manual_field,
                total,
            ),
            KeyCode::Down => select_next(
                &mut self.state.soft_delete_screen.selected_manual_field,
                total,
            ),
            KeyCode::Enter => {
                if let Some(field) = self
                    .manual_soft_delete_candidates()
                    .get(self.state.soft_delete_screen.selected_manual_field)
                    .cloned()
                {
                    self.state.preview = self.load_preview_for_selected_table(
                        SoftDeletePreference::PreferDeleteEndpoint,
                        Some(field.as_str()),
                    );
                    self.state.screen = Screen::CommandPreview;
                    self.state.command_preview_screen.selected_action =
                        CommandPreviewAction::ExecuteCommand;
                    self.state.last_message =
                        format!("Usaremos `{}` como columna de soft delete.", field);
                }
            }
            _ => {}
        }
    }

    fn handle_sql_preview(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.state.screen = Screen::SoftDeleteStrategy;
                self.state.last_message =
                    "Puedes elegir otra estrategia antes de ejecutar nada.".to_string();
            }
            KeyCode::Up => {
                self.state.sql_preview_screen.selected_action =
                    previous_sql_preview_action(self.state.sql_preview_screen.selected_action);
            }
            KeyCode::Down => {
                self.state.sql_preview_screen.selected_action =
                    next_sql_preview_action(self.state.sql_preview_screen.selected_action);
            }
            KeyCode::Enter => match self.state.sql_preview_screen.selected_action {
                SqlPreviewAction::ExecuteAndContinue => {
                    self.state.screen = Screen::CommandPreview;
                    self.state.command_preview_screen.selected_action =
                        CommandPreviewAction::ExecuteCommand;
                    self.state.last_message =
                        "ALTER TABLE confirmado para este flujo. La metadata se considera refrescada."
                            .to_string();
                }
                SqlPreviewAction::CopyAndContinue => {
                    self.state.screen = Screen::CommandPreview;
                    self.state.command_preview_screen.selected_action =
                        CommandPreviewAction::ExecuteCommand;
                    self.state.last_message =
                        "SQL marcado como copiado. Puedes ejecutarlo aparte y seguir al comando."
                            .to_string();
                }
                SqlPreviewAction::CopyAndStay => {
                    self.state.last_message =
                        "SQL marcado como copiado. Puedes ejecutarlo aparte cuando quieras."
                            .to_string();
                }
                SqlPreviewAction::Cancel => {
                    self.state.screen = Screen::SoftDeleteStrategy;
                    self.state.last_message =
                        "Operacion cancelada. La base sigue intacta.".to_string();
                }
            },
            _ => {}
        }
    }

    fn handle_command_preview(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.state.screen = if self
                    .state
                    .preview
                    .as_ref()
                    .and_then(|preview| preview.generated_sql.as_ref())
                    .is_some()
                {
                    Screen::SqlPreview
                } else {
                    Screen::ObjectDetails
                };
                self.state.last_message =
                    "Puedes revisar el paso anterior antes de ejecutar.".to_string();
            }
            KeyCode::Up => {
                self.state.command_preview_screen.selected_action = previous_command_preview_action(
                    self.state.command_preview_screen.selected_action,
                );
            }
            KeyCode::Down => {
                self.state.command_preview_screen.selected_action =
                    next_command_preview_action(self.state.command_preview_screen.selected_action);
            }
            KeyCode::Enter => match self.state.command_preview_screen.selected_action {
                CommandPreviewAction::ExecuteCommand => match self.run_generated_command() {
                    Ok(result) => {
                        self.state.process_result = Some(result);
                        self.state.process_result_screen.scroll = 0;
                        self.state.screen = Screen::ProcessResult;
                        self.state.last_message =
                            "Ejecucion completada. Ya puedes revisar stdout y stderr.".to_string();
                    }
                    Err(error) => {
                        self.state.process_result = Some(ProcessResult {
                            exit_code: -1,
                            stdout: String::new(),
                            stderr: error,
                            success: false,
                        });
                        self.state.process_result_screen.scroll = 0;
                        self.state.screen = Screen::ProcessResult;
                        self.state.last_message =
                            "La ejecucion del comando fallo antes de completar el proceso."
                                .to_string();
                    }
                },
                CommandPreviewAction::CopyAndMarkExternalExecution => {
                    self.state.process_result =
                        Some(mock_external_process_result(self.state.preview.as_ref()));
                    self.state.process_result_screen.scroll = 0;
                    self.state.screen = Screen::ProcessResult;
                    self.state.last_message =
                        "Comando marcado como copiado para ejecucion externa.".to_string();
                }
                CommandPreviewAction::CopyAndStay => {
                    self.state.last_message =
                        "Comando marcado como copiado al portapapeles virtual.".to_string();
                }
                CommandPreviewAction::Cancel => {
                    self.state.screen = Screen::ObjectExplorer;
                    self.state.last_message =
                        "Ejecucion cancelada. Regresaste al explorador de objetos.".to_string();
                }
            },
            _ => {}
        }
    }

    fn handle_process_result(&mut self, code: KeyCode) {
        let total_lines = self.process_result_line_count() as u16;
        match code {
            KeyCode::Up => {
                self.state.process_result_screen.scroll =
                    self.state.process_result_screen.scroll.saturating_sub(1);
            }
            KeyCode::Down => {
                self.state.process_result_screen.scroll = self
                    .state
                    .process_result_screen
                    .scroll
                    .saturating_add(1)
                    .min(total_lines.saturating_sub(1));
            }
            KeyCode::PageUp => {
                self.state.process_result_screen.scroll =
                    self.state.process_result_screen.scroll.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.state.process_result_screen.scroll = self
                    .state
                    .process_result_screen
                    .scroll
                    .saturating_add(10)
                    .min(total_lines.saturating_sub(1));
            }
            KeyCode::Home => {
                self.state.process_result_screen.scroll = 0;
            }
            KeyCode::End => {
                self.state.process_result_screen.scroll = total_lines.saturating_sub(1);
            }
            KeyCode::Esc | KeyCode::Enter => {
                self.state.screen = Screen::ObjectExplorer;
                self.state.last_message =
                    "El flujo termino. Puedes probar otra tabla o cambiar de motor.".to_string();
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(frame.area());

        let title = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                "api-from-db-cli",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("TUI del flujo principal de generacion"),
        ]))
        .block(Block::default().borders(Borders::ALL).title("Proyecto"));
        frame.render_widget(title, areas[0]);

        match self.state.screen {
            Screen::ConnectionSource => screens::connection_source::render(self, frame, areas[1]),
            Screen::EngineSelect => screens::engine_select::render(self, frame, areas[1]),
            Screen::ObjectExplorer => screens::object_explorer::render(self, frame, areas[1]),
            Screen::ObjectDetails => screens::object_details::render(self, frame, areas[1]),
            Screen::SoftDeleteStrategy => {
                screens::soft_delete_strategy::render(self, frame, areas[1])
            }
            Screen::ManualSoftDeleteField => {
                screens::manual_soft_delete_field::render(self, frame, areas[1])
            }
            Screen::SqlPreview => screens::sql_preview::render(self, frame, areas[1]),
            Screen::CommandPreview => screens::command_preview::render(self, frame, areas[1]),
            Screen::ProcessResult => screens::process_result::render(self, frame, areas[1]),
        }

        let footer = Paragraph::new(self.footer_text())
            .block(Block::default().borders(Borders::ALL).title("Ayuda"))
            .wrap(Wrap { trim: true });
        frame.render_widget(footer, areas[2]);
    }

    fn footer_text(&self) -> String {
        let mut parts = vec![
            "↑/↓ mover",
            "Enter confirmar/reintentar",
            "Esc regresar",
            "q salir",
        ];
        if !self.state.last_message.is_empty() {
            parts.push(self.state.last_message.as_str());
        }
        parts.join("  |  ")
    }

    fn load_preview_for_selected_table(
        &mut self,
        preference: SoftDeletePreference,
        manually_selected_field: Option<&str>,
    ) -> Option<GenerationPreview> {
        let selected = self.state.selected_db_object.as_ref()?.clone();
        let engine = self.state.engine?;

        let schema = match self.state.catalog.mode {
            CatalogMode::Mock => self.mock_table_schema(&selected),
            CatalogMode::Real => self.real_table_schema(&selected),
        };

        match schema {
            Ok(schema) => {
                Some(self.preview_from_schema(schema, preference, manually_selected_field))
            }
            Err(error) => {
                self.state.last_message = format!(
                    "No fue posible leer la estructura de {}.{}: {}",
                    selected.schema.as_deref().unwrap_or("<sin schema>"),
                    selected.name,
                    error
                );
                self.state.engine = Some(engine);
                None
            }
        }
    }

    fn preview_from_schema(
        &self,
        schema: TableSchema,
        preference: SoftDeletePreference,
        manually_selected_field: Option<&str>,
    ) -> GenerationPreview {
        let engine = self.state.engine.unwrap_or(DatabaseEngine::PostgreSql);
        let service = GenerationService::new(engine, self.generator_config_for_current_flow());
        service.preview_from_table(schema, preference, manually_selected_field)
    }

    fn mock_table_schema(&self, selected: &DatabaseObject) -> Result<TableSchema, String> {
        self.state
            .catalog
            .tables
            .iter()
            .find(|table| {
                table.schema.name == selected.name
                    && table.schema.schema == selected.schema.clone().unwrap_or_default()
            })
            .map(|table| table.schema.clone())
            .ok_or_else(|| "la tabla no existe en el catalogo mock".to_string())
    }

    fn real_table_schema(&self, selected: &DatabaseObject) -> Result<TableSchema, String> {
        let config = self
            .state
            .connection_config
            .as_ref()
            .ok_or_else(|| "no hay conexion real activa".to_string())?;
        let schema = selected
            .schema
            .as_deref()
            .ok_or_else(|| "el objeto no trae schema".to_string())?;

        let adapter = SqlServerAdapter;
        adapter
            .get_table_schema(config, schema, &selected.name)
            .map_err(|error| error.to_string())
    }

    pub(crate) fn manual_soft_delete_candidates(&self) -> Vec<String> {
        self.state
            .preview
            .as_ref()
            .map(|preview| {
                preview
                    .analyzed_schema
                    .columns
                    .iter()
                    .filter(|column| column.normalized_type.as_generator_token() == "datetime")
                    .map(|column| column.name.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    fn needs_soft_delete_decision(&self) -> bool {
        self.state
            .preview
            .as_ref()
            .map(|preview| preview.soft_delete.field_was_created)
            .unwrap_or(false)
    }

    pub(crate) fn generator_config_for_current_flow(&self) -> crate::domain::GeneratorConfig {
        self.state
            .connection_config
            .as_ref()
            .map(|config| config.generator.clone())
            .unwrap_or_else(|| crate::domain::GeneratorConfig {
                cmd: "gen.exe".to_string(),
                flags: vec!["--mvc".to_string()],
            })
    }

    pub(crate) fn build_process_result_lines(&self) -> Vec<Line<'_>> {
        let result = self.state.process_result.as_ref();
        let mut lines = vec![
            Line::from(format!(
                "Exit code: {}",
                result.map(|item| item.exit_code).unwrap_or(-1)
            )),
            Line::from(""),
            Line::from("STDOUT:"),
        ];
        lines.extend(split_render_lines(
            result.map(|item| item.stdout.as_str()).unwrap_or_default(),
        ));
        lines.push(Line::from(""));
        lines.push(Line::from("STDERR:"));
        lines.extend(split_render_lines(
            result.map(|item| item.stderr.as_str()).unwrap_or_default(),
        ));
        lines.push(Line::from(""));
        lines.push(Line::from(
            "Usa ↑/↓, PgUp/PgDn, Home/End para navegar. Enter o Esc para volver.",
        ));
        lines
    }

    fn process_result_line_count(&self) -> usize {
        self.build_process_result_lines().len()
    }

    fn run_generated_command(&self) -> Result<ProcessResult, String> {
        let preview = self
            .state
            .preview
            .as_ref()
            .ok_or_else(|| "No hay comando generado para ejecutar.".to_string())?;

        let runner = StdProcessRunner;
        runner
            .run(&preview.generated_command)
            .map_err(|error| error.to_string())
    }
}

pub(crate) fn selected_index<T: Copy + PartialEq, const N: usize>(
    options: [T; N],
    selected: T,
) -> usize {
    options
        .iter()
        .position(|option| *option == selected)
        .unwrap_or(0)
}

fn next_connection_source(selected: ConnectionSource) -> ConnectionSource {
    match selected {
        ConnectionSource::Manual => ConnectionSource::ConfigFile,
        ConnectionSource::ConfigFile => ConnectionSource::Manual,
    }
}

fn previous_connection_source(selected: ConnectionSource) -> ConnectionSource {
    next_connection_source(selected)
}

fn next_engine_option(selected: EngineOption) -> EngineOption {
    match selected {
        EngineOption::PostgreSql => EngineOption::SqlServer,
        EngineOption::SqlServer => EngineOption::PostgreSql,
    }
}

fn previous_engine_option(selected: EngineOption) -> EngineOption {
    next_engine_option(selected)
}

fn next_soft_delete_strategy(selected: SoftDeleteStrategy) -> SoftDeleteStrategy {
    match selected {
        SoftDeleteStrategy::CreateDeletedAt => SoftDeleteStrategy::UseExistingField,
        SoftDeleteStrategy::UseExistingField => SoftDeleteStrategy::ContinueWithoutDelete,
        SoftDeleteStrategy::ContinueWithoutDelete => SoftDeleteStrategy::CreateDeletedAt,
    }
}

fn previous_soft_delete_strategy(selected: SoftDeleteStrategy) -> SoftDeleteStrategy {
    match selected {
        SoftDeleteStrategy::CreateDeletedAt => SoftDeleteStrategy::ContinueWithoutDelete,
        SoftDeleteStrategy::UseExistingField => SoftDeleteStrategy::CreateDeletedAt,
        SoftDeleteStrategy::ContinueWithoutDelete => SoftDeleteStrategy::UseExistingField,
    }
}

fn next_sql_preview_action(selected: SqlPreviewAction) -> SqlPreviewAction {
    match selected {
        SqlPreviewAction::ExecuteAndContinue => SqlPreviewAction::CopyAndContinue,
        SqlPreviewAction::CopyAndContinue => SqlPreviewAction::CopyAndStay,
        SqlPreviewAction::CopyAndStay => SqlPreviewAction::Cancel,
        SqlPreviewAction::Cancel => SqlPreviewAction::ExecuteAndContinue,
    }
}

fn previous_sql_preview_action(selected: SqlPreviewAction) -> SqlPreviewAction {
    match selected {
        SqlPreviewAction::ExecuteAndContinue => SqlPreviewAction::Cancel,
        SqlPreviewAction::CopyAndContinue => SqlPreviewAction::ExecuteAndContinue,
        SqlPreviewAction::CopyAndStay => SqlPreviewAction::CopyAndContinue,
        SqlPreviewAction::Cancel => SqlPreviewAction::CopyAndStay,
    }
}

fn next_command_preview_action(selected: CommandPreviewAction) -> CommandPreviewAction {
    match selected {
        CommandPreviewAction::ExecuteCommand => CommandPreviewAction::CopyAndMarkExternalExecution,
        CommandPreviewAction::CopyAndMarkExternalExecution => CommandPreviewAction::CopyAndStay,
        CommandPreviewAction::CopyAndStay => CommandPreviewAction::Cancel,
        CommandPreviewAction::Cancel => CommandPreviewAction::ExecuteCommand,
    }
}

fn previous_command_preview_action(selected: CommandPreviewAction) -> CommandPreviewAction {
    match selected {
        CommandPreviewAction::ExecuteCommand => CommandPreviewAction::Cancel,
        CommandPreviewAction::CopyAndMarkExternalExecution => CommandPreviewAction::ExecuteCommand,
        CommandPreviewAction::CopyAndStay => CommandPreviewAction::CopyAndMarkExternalExecution,
        CommandPreviewAction::Cancel => CommandPreviewAction::CopyAndStay,
    }
}

fn initial_connection_message() -> String {
    match detect_generator_binary_state() {
        GeneratorBinaryState::Available { path } => format!(
            "Se encontro el generador en `{}`. Selecciona como quieres cargar la conexion.",
            path.display()
        ),
        GeneratorBinaryState::Missing { searched_paths } => format!(
            "No se encontro `gen` ni `gen.exe` en la raiz del proyecto. Colocalo en {} y presiona Enter para reintentar.",
            searched_paths
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(" o ")
        ),
        GeneratorBinaryState::Error(error) => format!(
            "No fue posible validar el generador en la raiz del proyecto: {}",
            error
        ),
    }
}

fn detect_generator_binary_state() -> GeneratorBinaryState {
    let current_dir = match env::current_dir() {
        Ok(path) => path,
        Err(error) => return GeneratorBinaryState::Error(error.to_string()),
    };

    let candidates = [current_dir.join("gen"), current_dir.join("gen.exe")];

    for candidate in &candidates {
        if candidate.is_file() {
            return GeneratorBinaryState::Available {
                path: candidate.clone(),
            };
        }
    }

    GeneratorBinaryState::Missing {
        searched_paths: candidates.into_iter().collect(),
    }
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

fn split_render_lines(content: &str) -> Vec<Line<'_>> {
    if content.is_empty() {
        return vec![Line::from("")];
    }

    content.split('\n').map(Line::from).collect()
}
pub(crate) fn render_selectable_list(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    items: &[ListItem<'_>],
    selected: usize,
) {
    let list = List::new(items.to_vec())
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");

    let mut state = ListState::default();
    if !items.is_empty() {
        state.select(Some(selected.min(items.len().saturating_sub(1))));
    }
    frame.render_stateful_widget(list, area, &mut state);
}

pub(crate) fn draw_menu(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    items: &[ListItem<'_>],
    selected: usize,
    helper: Option<&str>,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    render_selectable_list(frame, chunks[0], title, items, selected);

    let helper_text = helper.unwrap_or("Selecciona una opcion para continuar.");
    let is_alert = is_critical_helper_message(helper_text);
    let helper_block = if is_alert {
        Block::default()
            .borders(Borders::ALL)
            .title("Atencion")
            .style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            )
    } else {
        Block::default().borders(Borders::ALL).title("Detalle")
    };
    let helper_widget = Paragraph::new(helper_text)
        .block(helper_block)
        .style(if is_alert {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        })
        .wrap(Wrap { trim: true });
    frame.render_widget(helper_widget, chunks[1]);
}

fn is_critical_helper_message(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("no se encontro")
        || lower.contains("no fue posible")
        || lower.contains("faltante")
        || lower.contains("error")
}

pub(crate) fn two_column_layout(area: Rect) -> [Rect; 2] {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);
    [chunks[0], chunks[1]]
}

fn select_next(selected: &mut usize, total: usize) {
    if total == 0 {
        *selected = 0;
    } else {
        *selected = (*selected + 1) % total;
    }
}

fn select_previous(selected: &mut usize, total: usize) {
    if total == 0 {
        *selected = 0;
    } else if *selected == 0 {
        *selected = total - 1;
    } else {
        *selected -= 1;
    }
}

pub(crate) fn object_type_label(object_type: DatabaseObjectType) -> &'static str {
    match object_type {
        DatabaseObjectType::Table => "table",
        DatabaseObjectType::Function => "function",
        DatabaseObjectType::StoredProcedure => "stored procedure",
    }
}

fn mock_external_process_result(preview: Option<&GenerationPreview>) -> ProcessResult {
    let command = preview
        .map(|item| item.generated_command.raw_command.clone())
        .unwrap_or_else(|| "gen.exe demo id:int".to_string());

    ProcessResult {
        exit_code: 0,
        stdout: format!(
            "Comando copiado para ejecucion externa:\n{command}\nPuedes correrlo en otra herramienta y volver despues."
        ),
        stderr: String::new(),
        success: true,
    }
}

fn mock_catalog(engine: DatabaseEngine) -> Catalog {
    match engine {
        DatabaseEngine::PostgreSql => Catalog {
            mode: CatalogMode::Mock,
            objects: vec![
                DatabaseObject {
                    name: "users".to_string(),
                    schema: Some("public".to_string()),
                    object_type: DatabaseObjectType::Table,
                    engine,
                },
                DatabaseObject {
                    name: "orders".to_string(),
                    schema: Some("sales".to_string()),
                    object_type: DatabaseObjectType::Table,
                    engine,
                },
                DatabaseObject {
                    name: "calculate_total".to_string(),
                    schema: Some("sales".to_string()),
                    object_type: DatabaseObjectType::Function,
                    engine,
                },
            ],
            tables: vec![
                MockTable {
                    schema: TableSchema::new(
                        "public",
                        "users",
                        vec![
                            ColumnSchema::new("id", "integer", false, 1),
                            ColumnSchema::new("name", "varchar(50)", false, 2),
                            ColumnSchema::new("email", "varchar(255)", false, 3),
                            ColumnSchema::new("created_at", "timestamp", false, 4),
                            ColumnSchema::new("updated_at", "timestamp", true, 5),
                        ],
                        vec!["id".to_string()],
                    ),
                },
                MockTable {
                    schema: TableSchema::new(
                        "sales",
                        "orders",
                        vec![
                            ColumnSchema::new("id", "bigint", false, 1),
                            ColumnSchema::new("customer_name", "text", false, 2),
                            ColumnSchema::new("total", "numeric(10,2)", false, 3),
                            ColumnSchema::new("deleted_at", "timestamptz", true, 4),
                        ],
                        vec!["id".to_string()],
                    ),
                },
            ],
        },
        DatabaseEngine::SqlServer => Catalog {
            mode: CatalogMode::Mock,
            objects: vec![
                DatabaseObject {
                    name: "Employees".to_string(),
                    schema: Some("dbo".to_string()),
                    object_type: DatabaseObjectType::Table,
                    engine,
                },
                DatabaseObject {
                    name: "Invoices".to_string(),
                    schema: Some("billing".to_string()),
                    object_type: DatabaseObjectType::Table,
                    engine,
                },
                DatabaseObject {
                    name: "CreateInvoice".to_string(),
                    schema: Some("billing".to_string()),
                    object_type: DatabaseObjectType::StoredProcedure,
                    engine,
                },
            ],
            tables: vec![
                MockTable {
                    schema: TableSchema::new(
                        "dbo",
                        "Employees",
                        vec![
                            ColumnSchema::new("id", "int", false, 1),
                            ColumnSchema::new("first_name", "nvarchar(100)", false, 2),
                            ColumnSchema::new("last_name", "nvarchar(100)", false, 3),
                            ColumnSchema::new("deactivated_on", "datetime2", true, 4),
                        ],
                        vec!["id".to_string()],
                    ),
                },
                MockTable {
                    schema: TableSchema::new(
                        "billing",
                        "Invoices",
                        vec![
                            ColumnSchema::new("id", "bigint", false, 1),
                            ColumnSchema::new("folio", "varchar(30)", false, 2),
                            ColumnSchema::new("issued_at", "datetimeoffset", false, 3),
                            ColumnSchema::new("deleted_at", "datetime2", true, 4),
                        ],
                        vec!["id".to_string()],
                    ),
                },
            ],
        },
    }
}
