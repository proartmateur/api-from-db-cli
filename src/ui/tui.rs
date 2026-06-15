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
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal};

use crate::adapters::config::ConfigLoader;
use crate::app::generation_service::{GenerationPreview, GenerationService};
use crate::domain::{
    ColumnSchema, DatabaseEngine, DatabaseObject, DatabaseObjectType, SoftDeletePreference,
    TableSchema,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
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
enum ConnectionSource {
    Manual,
    ConfigFile,
}

impl ConnectionSource {
    fn label(self) -> &'static str {
        match self {
            Self::Manual => "Captura manual",
            Self::ConfigFile => "Archivo de configuracion",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SoftDeleteStrategy {
    CreateDeletedAt,
    UseExistingField,
    ContinueWithoutDelete,
}

impl SoftDeleteStrategy {
    fn label(self) -> &'static str {
        match self {
            Self::CreateDeletedAt => "Crear campo deleted_at",
            Self::UseExistingField => "Usar columna existente",
            Self::ContinueWithoutDelete => "Continuar sin endpoint delete",
        }
    }
}

#[derive(Debug, Clone)]
struct MockTable {
    schema: TableSchema,
}

#[derive(Debug, Clone)]
struct MockCatalog {
    objects: Vec<DatabaseObject>,
    tables: Vec<MockTable>,
}

#[derive(Debug, Clone)]
struct MockProcessResult {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Clone)]
struct AppState {
    screen: Screen,
    selected_connection_source: usize,
    selected_engine: usize,
    selected_object: usize,
    selected_soft_delete_strategy: usize,
    selected_manual_field: usize,
    selected_action: usize,
    connection_source: Option<ConnectionSource>,
    engine: Option<DatabaseEngine>,
    catalog: MockCatalog,
    selected_db_object: Option<DatabaseObject>,
    preview: Option<GenerationPreview>,
    process_result: Option<MockProcessResult>,
    config_file_path: Option<String>,
    last_message: String,
}

pub struct TuiApp {
    state: AppState,
}

impl TuiApp {
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
                    Ok(config) => {
                        self.state.engine = Some(config.engine);
                        self.state.catalog = mock_catalog(config.engine);
                        self.state.selected_object = 0;
                        self.state.preview = None;
                        self.state.process_result = None;
                        self.state.screen = Screen::ObjectExplorer;
                        self.state.last_message = format!(
                            "Configuracion cargada desde `{}`. Se activo el flujo mock para {}.",
                            path, config.engine
                        );
                    }
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

    pub fn new() -> Self {
        Self {
            state: AppState {
                screen: Screen::ConnectionSource,
                selected_connection_source: 0,
                selected_engine: 0,
                selected_object: 0,
                selected_soft_delete_strategy: 0,
                selected_manual_field: 0,
                selected_action: 0,
                connection_source: None,
                engine: None,
                catalog: mock_catalog(DatabaseEngine::PostgreSql),
                selected_db_object: None,
                preview: None,
                process_result: None,
                config_file_path: None,
                last_message: "Selecciona como quieres cargar la conexion mock.".to_string(),
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

    fn handle_connection_source(&mut self, code: KeyCode) {
        let total = 2;
        match code {
            KeyCode::Up => select_previous(&mut self.state.selected_connection_source, total),
            KeyCode::Down => select_next(&mut self.state.selected_connection_source, total),
            KeyCode::Enter => match self.state.selected_connection_source {
                0 => {
                    self.state.connection_source = Some(ConnectionSource::Manual);
                    self.state.screen = Screen::EngineSelect;
                    self.state.last_message =
                        "Conexion mock preparada. Ahora elige el motor de base de datos."
                            .to_string();
                }
                _ => self.handle_config_file_selection(),
            },
            _ => {}
        }
    }

    fn handle_engine_select(&mut self, code: KeyCode) {
        let total = 2;
        match code {
            KeyCode::Esc => {
                self.state.screen = Screen::ConnectionSource;
                self.state.last_message =
                    "Regresaste a la seleccion del origen de conexion.".to_string();
            }
            KeyCode::Up => select_previous(&mut self.state.selected_engine, total),
            KeyCode::Down => select_next(&mut self.state.selected_engine, total),
            KeyCode::Enter => {
                let engine = match self.state.selected_engine {
                    0 => DatabaseEngine::PostgreSql,
                    _ => DatabaseEngine::SqlServer,
                };
                self.state.engine = Some(engine);
                self.state.catalog = mock_catalog(engine);
                self.state.selected_object = 0;
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
                self.state.screen = Screen::EngineSelect;
                self.state.last_message =
                    "Puedes cambiar de motor sin tocar infraestructura real.".to_string();
            }
            KeyCode::Up => select_previous(&mut self.state.selected_object, total),
            KeyCode::Down => select_next(&mut self.state.selected_object, total),
            KeyCode::Enter => {
                if let Some(object) = self
                    .state
                    .catalog
                    .objects
                    .get(self.state.selected_object)
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
                        self.state.preview = self.generate_preview_for_selected_table(
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
                self.state.last_message = "Regresaste al explorador de objetos mock.".to_string();
            }
            KeyCode::Enter if selected.object_type == DatabaseObjectType::Table => {
                if self.needs_soft_delete_decision() {
                    self.state.screen = Screen::SoftDeleteStrategy;
                    self.state.selected_soft_delete_strategy = 0;
                    self.state.last_message =
                        "No existe deleted_at compatible. Decide como quieres resolver soft delete."
                            .to_string();
                } else {
                    self.state.screen = Screen::CommandPreview;
                    self.state.selected_action = 0;
                    self.state.last_message =
                        "La tabla ya tiene suficiente metadata para construir el comando."
                            .to_string();
                }
            }
            _ => {}
        }
    }

    fn handle_soft_delete_strategy(&mut self, code: KeyCode) {
        let total = 3;
        match code {
            KeyCode::Esc => {
                self.state.screen = Screen::ObjectDetails;
                self.state.last_message =
                    "Puedes revisar de nuevo la tabla antes de decidir.".to_string();
            }
            KeyCode::Up => select_previous(&mut self.state.selected_soft_delete_strategy, total),
            KeyCode::Down => select_next(&mut self.state.selected_soft_delete_strategy, total),
            KeyCode::Enter => match self.selected_strategy() {
                SoftDeleteStrategy::CreateDeletedAt => {
                    self.state.preview = self.generate_preview_for_selected_table(
                        SoftDeletePreference::PreferDeleteEndpoint,
                        None,
                    );
                    self.state.screen = Screen::SqlPreview;
                    self.state.selected_action = 0;
                    self.state.last_message =
                        "Se genero el ALTER TABLE mock para revisar antes de continuar."
                            .to_string();
                }
                SoftDeleteStrategy::UseExistingField => {
                    self.state.selected_manual_field = 0;
                    self.state.screen = Screen::ManualSoftDeleteField;
                    self.state.last_message =
                        "Selecciona una columna datetime existente para soft delete.".to_string();
                }
                SoftDeleteStrategy::ContinueWithoutDelete => {
                    self.state.preview = self.generate_preview_for_selected_table(
                        SoftDeletePreference::SkipDeleteEndpoint,
                        None,
                    );
                    self.state.screen = Screen::CommandPreview;
                    self.state.selected_action = 0;
                    self.state.last_message =
                        "Seguimos sin endpoint delete para esta API mock.".to_string();
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
            KeyCode::Up => select_previous(&mut self.state.selected_manual_field, total),
            KeyCode::Down => select_next(&mut self.state.selected_manual_field, total),
            KeyCode::Enter => {
                if let Some(field) = self
                    .manual_soft_delete_candidates()
                    .get(self.state.selected_manual_field)
                    .cloned()
                {
                    self.state.preview = self.generate_preview_for_selected_table(
                        SoftDeletePreference::PreferDeleteEndpoint,
                        Some(field.as_str()),
                    );
                    self.state.screen = Screen::CommandPreview;
                    self.state.selected_action = 0;
                    self.state.last_message = format!(
                        "Usaremos `{}` como columna de soft delete para el mock.",
                        field
                    );
                }
            }
            _ => {}
        }
    }

    fn handle_sql_preview(&mut self, code: KeyCode) {
        let total = 4;
        match code {
            KeyCode::Esc => {
                self.state.screen = Screen::SoftDeleteStrategy;
                self.state.last_message =
                    "Puedes elegir otra estrategia antes de ejecutar nada.".to_string();
            }
            KeyCode::Up => select_previous(&mut self.state.selected_action, total),
            KeyCode::Down => select_next(&mut self.state.selected_action, total),
            KeyCode::Enter => match self.state.selected_action {
                0 => {
                    self.state.screen = Screen::CommandPreview;
                    self.state.selected_action = 0;
                    self.state.last_message =
                        "ALTER TABLE mock confirmado. La metadata se considera refrescada."
                            .to_string();
                }
                1 => {
                    self.state.screen = Screen::CommandPreview;
                    self.state.selected_action = 0;
                    self.state.last_message =
                        "SQL mock marcado como copiado. Puedes ejecutarlo aparte y seguir al comando."
                            .to_string();
                }
                2 => {
                    self.state.last_message =
                        "SQL mock marcado como copiado. Puedes ejecutarlo aparte cuando quieras."
                            .to_string();
                }
                _ => {
                    self.state.screen = Screen::SoftDeleteStrategy;
                    self.state.last_message =
                        "Operacion cancelada. La base mock sigue intacta.".to_string();
                }
            },
            _ => {}
        }
    }

    fn handle_command_preview(&mut self, code: KeyCode) {
        let total = 4;
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
            KeyCode::Up => select_previous(&mut self.state.selected_action, total),
            KeyCode::Down => select_next(&mut self.state.selected_action, total),
            KeyCode::Enter => match self.state.selected_action {
                0 => {
                    self.state.process_result =
                        Some(mock_process_result(self.state.preview.as_ref()));
                    self.state.screen = Screen::ProcessResult;
                    self.state.last_message =
                        "Ejecucion mock completada. Ya puedes revisar stdout y stderr.".to_string();
                }
                1 => {
                    self.state.process_result =
                        Some(mock_external_process_result(self.state.preview.as_ref()));
                    self.state.screen = Screen::ProcessResult;
                    self.state.last_message =
                        "Comando mock marcado como copiado para ejecucion externa.".to_string();
                }
                2 => {
                    self.state.last_message =
                        "Comando mock marcado como copiado al portapapeles virtual.".to_string();
                }
                _ => {
                    self.state.screen = Screen::ObjectExplorer;
                    self.state.last_message =
                        "Ejecucion cancelada. Regresaste al explorador de objetos.".to_string();
                }
            },
            _ => {}
        }
    }

    fn handle_process_result(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Enter => {
                self.state.screen = Screen::ObjectExplorer;
                self.state.last_message =
                    "El flujo mock termino. Puedes probar otra tabla o cambiar de motor."
                        .to_string();
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
            Line::from("TUI mock del flujo principal de generacion"),
        ]))
        .block(Block::default().borders(Borders::ALL).title("Proyecto"));
        frame.render_widget(title, areas[0]);

        match self.state.screen {
            Screen::ConnectionSource => self.draw_connection_source(frame, areas[1]),
            Screen::EngineSelect => self.draw_engine_select(frame, areas[1]),
            Screen::ObjectExplorer => self.draw_object_explorer(frame, areas[1]),
            Screen::ObjectDetails => self.draw_object_details(frame, areas[1]),
            Screen::SoftDeleteStrategy => self.draw_soft_delete_strategy(frame, areas[1]),
            Screen::ManualSoftDeleteField => self.draw_manual_soft_delete_field(frame, areas[1]),
            Screen::SqlPreview => self.draw_sql_preview(frame, areas[1]),
            Screen::CommandPreview => self.draw_command_preview(frame, areas[1]),
            Screen::ProcessResult => self.draw_process_result(frame, areas[1]),
        }

        let footer = Paragraph::new(self.footer_text())
            .block(Block::default().borders(Borders::ALL).title("Ayuda"))
            .wrap(Wrap { trim: true });
        frame.render_widget(footer, areas[2]);
    }

    fn draw_connection_source(&self, frame: &mut Frame, area: Rect) {
        let options = [ConnectionSource::Manual, ConnectionSource::ConfigFile];
        let items = options
            .iter()
            .map(|source| ListItem::new(source.label()))
            .collect::<Vec<_>>();

        let helper_message = match self.state.config_file_path.as_deref() {
            Some(path) => format!(
                "{}\n\nRuta actual del archivo: {}",
                self.state.last_message, path
            ),
            None => self.state.last_message.clone(),
        };

        draw_menu(
            frame,
            area,
            "Origen de Conexion",
            &items,
            self.state.selected_connection_source,
            Some(helper_message.as_str()),
        );
    }

    fn draw_engine_select(&self, frame: &mut Frame, area: Rect) {
        let items = vec![ListItem::new("PostgreSQL"), ListItem::new("SQL Server")];
        draw_menu(
            frame,
            area,
            "Motor",
            &items,
            self.state.selected_engine,
            Some(self.state.last_message.as_str()),
        );
    }

    fn draw_object_explorer(&self, frame: &mut Frame, area: Rect) {
        let chunks = two_column_layout(area);
        let items = self
            .state
            .catalog
            .objects
            .iter()
            .map(|object| {
                let schema = object.schema.as_deref().unwrap_or("<sin schema>");
                ListItem::new(format!(
                    "[{schema}] {} {}",
                    object_type_label(object.object_type),
                    object.name
                ))
            })
            .collect::<Vec<_>>();

        render_selectable_list(
            frame,
            chunks[0],
            "Objetos Disponibles",
            &items,
            self.state.selected_object,
        );

        let right_text = vec![
            Line::from("Conexion mock exitosa."),
            Line::from(format!(
                "Origen: {}",
                self.state
                    .connection_source
                    .map(ConnectionSource::label)
                    .unwrap_or("Pendiente")
            )),
            Line::from(format!(
                "Motor: {}",
                self.state
                    .engine
                    .map(|engine| engine.to_string())
                    .unwrap_or_else(|| "pendiente".to_string())
            )),
            Line::from(""),
            Line::from("Incluye tablas, funciones o stored procedures simulados."),
            Line::from(self.state.last_message.clone()),
        ];
        let details = Paragraph::new(Text::from(right_text))
            .block(Block::default().borders(Borders::ALL).title("Contexto"))
            .wrap(Wrap { trim: true });
        frame.render_widget(details, chunks[1]);
    }

    fn draw_object_details(&self, frame: &mut Frame, area: Rect) {
        let chunks = two_column_layout(area);
        let Some(selected) = self.state.selected_db_object.as_ref() else {
            return;
        };

        let left = if selected.object_type == DatabaseObjectType::Table {
            let preview = self.state.preview.as_ref();
            let mut lines = vec![Line::from(format!(
                "Tabla: {}.{}",
                selected.schema.as_deref().unwrap_or("<sin schema>"),
                selected.name
            ))];
            lines.push(Line::from(""));
            lines.push(Line::from("Columnas detectadas:"));

            if let Some(preview) = preview {
                for column in &preview.analyzed_schema.columns {
                    let marker = if column.is_primary_key { "PK" } else { "  " };
                    lines.push(Line::from(format!(
                        "{marker} {}:{} ({})",
                        column.name, column.normalized_type, column.db_type
                    )));
                }
            }

            Paragraph::new(Text::from(lines))
                .block(Block::default().borders(Borders::ALL).title("Schema"))
                .wrap(Wrap { trim: true })
        } else {
            Paragraph::new(Text::from(vec![
                Line::from(format!(
                    "{} seleccionado: {}",
                    object_type_label(selected.object_type),
                    selected.name
                )),
                Line::from(""),
                Line::from("La exploracion de metadata esta simulada."),
                Line::from("La generacion de API desde este tipo de objeto aun no esta soportada."),
            ]))
            .block(Block::default().borders(Borders::ALL).title("Metadata"))
            .wrap(Wrap { trim: true })
        };
        frame.render_widget(left, chunks[0]);

        let right_lines = if selected.object_type == DatabaseObjectType::Table {
            vec![
                Line::from("Siguiente paso:"),
                Line::from("Presiona Enter para resolver soft delete y construir el comando."),
                Line::from(""),
                Line::from(self.state.last_message.clone()),
            ]
        } else {
            vec![
                Line::from("Objeto no generable en esta version."),
                Line::from("Presiona Esc para volver al explorador."),
                Line::from(""),
                Line::from(self.state.last_message.clone()),
            ]
        };
        let right = Paragraph::new(Text::from(right_lines))
            .block(Block::default().borders(Borders::ALL).title("Accion"))
            .wrap(Wrap { trim: true });
        frame.render_widget(right, chunks[1]);
    }

    fn draw_soft_delete_strategy(&self, frame: &mut Frame, area: Rect) {
        let items = [
            SoftDeleteStrategy::CreateDeletedAt,
            SoftDeleteStrategy::UseExistingField,
            SoftDeleteStrategy::ContinueWithoutDelete,
        ]
        .iter()
        .map(|strategy| ListItem::new(strategy.label()))
        .collect::<Vec<_>>();

        draw_menu(
            frame,
            area,
            "Resolver Soft Delete",
            &items,
            self.state.selected_soft_delete_strategy,
            Some(self.state.last_message.as_str()),
        );
    }

    fn draw_manual_soft_delete_field(&self, frame: &mut Frame, area: Rect) {
        let candidates = self.manual_soft_delete_candidates();
        let items = candidates
            .iter()
            .map(|field| ListItem::new(field.clone()))
            .collect::<Vec<_>>();
        draw_menu(
            frame,
            area,
            "Columnas Datetime Disponibles",
            &items,
            self.state.selected_manual_field,
            Some("Estas columnas ya existen y pueden mapearse como delete_at."),
        );
    }

    fn draw_sql_preview(&self, frame: &mut Frame, area: Rect) {
        let chunks = two_column_layout(area);
        let preview = self.state.preview.as_ref();
        let sql = preview
            .and_then(|item| item.generated_sql.as_ref())
            .map(|item| item.sql.clone())
            .unwrap_or_else(|| "No hay SQL generado.".to_string());

        let sql_widget = Paragraph::new(sql)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Preview de ALTER TABLE"),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(sql_widget, chunks[0]);

        let actions = vec![
            ListItem::new("Ejecutar ALTER TABLE mock y continuar"),
            ListItem::new("Copiar SQL y continuar al comando"),
            ListItem::new("Copiar SQL y quedarse aqui"),
            ListItem::new("Cancelar"),
        ];
        render_selectable_list(
            frame,
            chunks[1],
            "Acciones",
            &actions,
            self.state.selected_action,
        );
    }

    fn draw_command_preview(&self, frame: &mut Frame, area: Rect) {
        let chunks = two_column_layout(area);
        let preview = self.state.preview.as_ref();
        let command = preview
            .map(|item| item.generated_command.raw_command.clone())
            .unwrap_or_else(|| "No hay comando generado.".to_string());

        let lines = vec![
            Line::from("Comando listo para el generador mock:"),
            Line::from(""),
            Line::from(command),
            Line::from(""),
            Line::from(
                preview
                    .map(|item| format!("Soft delete: {}", item.soft_delete.summary()))
                    .unwrap_or_else(|| "Soft delete no disponible".to_string()),
            ),
        ];
        let command_widget = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Preview de Comando"),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(command_widget, chunks[0]);

        let actions = vec![
            ListItem::new("Ejecutar comando mock"),
            ListItem::new("Copiar comando y marcar ejecucion externa"),
            ListItem::new("Copiar comando y quedarse aqui"),
            ListItem::new("Cancelar"),
        ];
        render_selectable_list(
            frame,
            chunks[1],
            "Acciones",
            &actions,
            self.state.selected_action,
        );
    }

    fn draw_process_result(&self, frame: &mut Frame, area: Rect) {
        let result = self.state.process_result.as_ref();
        let lines = vec![
            Line::from(format!(
                "Exit code: {}",
                result.map(|item| item.exit_code).unwrap_or(-1)
            )),
            Line::from(""),
            Line::from("STDOUT:"),
            Line::from(result.map(|item| item.stdout.clone()).unwrap_or_default()),
            Line::from(""),
            Line::from("STDERR:"),
            Line::from(result.map(|item| item.stderr.clone()).unwrap_or_default()),
            Line::from(""),
            Line::from("Presiona Enter o Esc para volver al explorador."),
        ];
        let widget = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Resultado del Generador Mock"),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(Clear, area);
        frame.render_widget(widget, area);
    }

    fn footer_text(&self) -> String {
        let mut parts = vec!["↑/↓ mover", "Enter confirmar", "Esc regresar", "q salir"];
        if !self.state.last_message.is_empty() {
            parts.push(self.state.last_message.as_str());
        }
        parts.join("  |  ")
    }

    fn selected_strategy(&self) -> SoftDeleteStrategy {
        match self.state.selected_soft_delete_strategy {
            0 => SoftDeleteStrategy::CreateDeletedAt,
            1 => SoftDeleteStrategy::UseExistingField,
            _ => SoftDeleteStrategy::ContinueWithoutDelete,
        }
    }

    fn generate_preview_for_selected_table(
        &self,
        preference: SoftDeletePreference,
        manually_selected_field: Option<&str>,
    ) -> Option<GenerationPreview> {
        let engine = self.state.engine?;
        let selected = self.state.selected_db_object.as_ref()?;
        let mock_table = self.state.catalog.tables.iter().find(|table| {
            table.schema.name == selected.name
                && table.schema.schema == selected.schema.clone().unwrap_or_default()
        })?;

        let service = GenerationService::new(engine, mock_generator_executable(engine));
        Some(service.preview_from_table(
            mock_table.schema.clone(),
            preference,
            manually_selected_field,
        ))
    }

    fn manual_soft_delete_candidates(&self) -> Vec<String> {
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
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

fn render_selectable_list(
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
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");

    let mut state = ListState::default();
    if !items.is_empty() {
        state.select(Some(selected.min(items.len().saturating_sub(1))));
    }
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_menu(
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
    let helper_widget = Paragraph::new(helper_text)
        .block(Block::default().borders(Borders::ALL).title("Detalle"))
        .wrap(Wrap { trim: true });
    frame.render_widget(helper_widget, chunks[1]);
}

fn two_column_layout(area: Rect) -> [Rect; 2] {
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

fn object_type_label(object_type: DatabaseObjectType) -> &'static str {
    match object_type {
        DatabaseObjectType::Table => "table",
        DatabaseObjectType::Function => "function",
        DatabaseObjectType::StoredProcedure => "stored procedure",
    }
}

fn mock_generator_executable(engine: DatabaseEngine) -> &'static str {
    match engine {
        DatabaseEngine::PostgreSql => "gen.exe",
        DatabaseEngine::SqlServer => "python-gen",
    }
}

fn mock_process_result(preview: Option<&GenerationPreview>) -> MockProcessResult {
    let command = preview
        .map(|item| item.generated_command.raw_command.clone())
        .unwrap_or_else(|| "gen.exe demo id:int".to_string());

    MockProcessResult {
        exit_code: 0,
        stdout: format!(
            "Ejecutando: {command}\nAPI generada correctamente.\nModels, services y routers creados."
        ),
        stderr: String::new(),
    }
}

fn mock_external_process_result(preview: Option<&GenerationPreview>) -> MockProcessResult {
    let command = preview
        .map(|item| item.generated_command.raw_command.clone())
        .unwrap_or_else(|| "gen.exe demo id:int".to_string());

    MockProcessResult {
        exit_code: 0,
        stdout: format!(
            "Comando copiado para ejecucion externa:\n{command}\nPuedes correrlo en otra herramienta y volver despues."
        ),
        stderr: String::new(),
    }
}

fn mock_catalog(engine: DatabaseEngine) -> MockCatalog {
    match engine {
        DatabaseEngine::PostgreSql => MockCatalog {
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
        DatabaseEngine::SqlServer => MockCatalog {
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
