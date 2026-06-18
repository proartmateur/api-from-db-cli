use std::env;
use std::io::{self, Stdout};
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
    ConnectionConfig, DatabaseEngine, DatabaseObject, DatabaseObjectType, ProcessResult,
    SoftDeletePreference, TableSchema,
};
use crate::ui::handlers;
use crate::ui::mocks::catalog_fn;
use crate::ui::screens;
use crate::ui::state::{
    AppState, Catalog, CatalogMode, CommandPreviewAction, CommandPreviewScreenState,
    ConnectionSource, ConnectionSourceScreenState, EngineOption, EngineSelectScreenState,
    GeneratorBinaryState, ObjectExplorerScreenState, ProcessResultScreenState, Screen,
    SoftDeleteScreenState, SoftDeleteStrategy, SqlPreviewAction, SqlPreviewScreenState,
};

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
                catalog: catalog_fn(DatabaseEngine::PostgreSql),
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
            Screen::ConnectionSource => handlers::connection_source::handle(self, code),
            Screen::EngineSelect => handlers::engine_select::handle(self, code),
            Screen::ObjectExplorer => handlers::object_explorer::handle(self, code),
            Screen::ObjectDetails => handlers::object_details::handle(self, code),
            Screen::SoftDeleteStrategy => handlers::soft_delete_strategy::handle(self, code),
            Screen::ManualSoftDeleteField => handlers::manual_soft_delete_field::handle(self, code),
            Screen::SqlPreview => handlers::sql_preview::handle(self, code),
            Screen::CommandPreview => handlers::command_preview::handle(self, code),
            Screen::ProcessResult => handlers::process_result::handle(self, code),
        }
    }

    fn refresh_generator_binary_state(&mut self) {
        self.state.generator_binary_state = detect_generator_binary_state();
    }

    pub(crate) fn ensure_generator_binary_ready(&mut self) -> bool {
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

    pub(crate) fn handle_config_file_selection(&mut self) {
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
                self.state.catalog = catalog_fn(DatabaseEngine::PostgreSql);
                self.state.screen = Screen::ObjectExplorer;
                self.state.last_message = format!(
                    "Configuracion cargada desde `{}`. PostgreSQL aun usa catalogo mock mientras conectamos su adapter real.",
                    path
                );
            }
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

    pub(crate) fn load_preview_for_selected_table(
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

    pub(crate) fn needs_soft_delete_decision(&self) -> bool {
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

    pub(crate) fn process_result_line_count(&self) -> usize {
        self.build_process_result_lines().len()
    }

    pub(crate) fn run_generated_command(&self) -> Result<ProcessResult, String> {
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

pub(crate) fn next_connection_source(selected: ConnectionSource) -> ConnectionSource {
    match selected {
        ConnectionSource::Manual => ConnectionSource::ConfigFile,
        ConnectionSource::ConfigFile => ConnectionSource::Manual,
    }
}

pub(crate) fn previous_connection_source(selected: ConnectionSource) -> ConnectionSource {
    next_connection_source(selected)
}

pub(crate) fn next_engine_option(selected: EngineOption) -> EngineOption {
    match selected {
        EngineOption::PostgreSql => EngineOption::SqlServer,
        EngineOption::SqlServer => EngineOption::PostgreSql,
    }
}

pub(crate) fn previous_engine_option(selected: EngineOption) -> EngineOption {
    next_engine_option(selected)
}

pub(crate) fn next_soft_delete_strategy(selected: SoftDeleteStrategy) -> SoftDeleteStrategy {
    match selected {
        SoftDeleteStrategy::CreateDeletedAt => SoftDeleteStrategy::UseExistingField,
        SoftDeleteStrategy::UseExistingField => SoftDeleteStrategy::ContinueWithoutDelete,
        SoftDeleteStrategy::ContinueWithoutDelete => SoftDeleteStrategy::CreateDeletedAt,
    }
}

pub(crate) fn previous_soft_delete_strategy(selected: SoftDeleteStrategy) -> SoftDeleteStrategy {
    match selected {
        SoftDeleteStrategy::CreateDeletedAt => SoftDeleteStrategy::ContinueWithoutDelete,
        SoftDeleteStrategy::UseExistingField => SoftDeleteStrategy::CreateDeletedAt,
        SoftDeleteStrategy::ContinueWithoutDelete => SoftDeleteStrategy::UseExistingField,
    }
}

pub(crate) fn next_sql_preview_action(selected: SqlPreviewAction) -> SqlPreviewAction {
    match selected {
        SqlPreviewAction::ExecuteAndContinue => SqlPreviewAction::CopyAndContinue,
        SqlPreviewAction::CopyAndContinue => SqlPreviewAction::CopyAndStay,
        SqlPreviewAction::CopyAndStay => SqlPreviewAction::Cancel,
        SqlPreviewAction::Cancel => SqlPreviewAction::ExecuteAndContinue,
    }
}

pub(crate) fn previous_sql_preview_action(selected: SqlPreviewAction) -> SqlPreviewAction {
    match selected {
        SqlPreviewAction::ExecuteAndContinue => SqlPreviewAction::Cancel,
        SqlPreviewAction::CopyAndContinue => SqlPreviewAction::ExecuteAndContinue,
        SqlPreviewAction::CopyAndStay => SqlPreviewAction::CopyAndContinue,
        SqlPreviewAction::Cancel => SqlPreviewAction::CopyAndStay,
    }
}

pub(crate) fn next_command_preview_action(selected: CommandPreviewAction) -> CommandPreviewAction {
    match selected {
        CommandPreviewAction::ExecuteCommand => CommandPreviewAction::CopyAndMarkExternalExecution,
        CommandPreviewAction::CopyAndMarkExternalExecution => CommandPreviewAction::CopyAndStay,
        CommandPreviewAction::CopyAndStay => CommandPreviewAction::Cancel,
        CommandPreviewAction::Cancel => CommandPreviewAction::ExecuteCommand,
    }
}

pub(crate) fn previous_command_preview_action(
    selected: CommandPreviewAction,
) -> CommandPreviewAction {
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

pub(crate) fn select_next(selected: &mut usize, total: usize) {
    if total == 0 {
        *selected = 0;
    } else {
        *selected = (*selected + 1) % total;
    }
}

pub(crate) fn select_previous(selected: &mut usize, total: usize) {
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
