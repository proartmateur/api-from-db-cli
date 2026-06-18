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

use crate::app::generation_service::GenerationPreview;
use crate::domain::{DatabaseEngine, DatabaseObjectType, ProcessResult, SoftDeletePreference};
use crate::ui::handlers;
use crate::ui::mocks::catalog_fn;
use crate::ui::screens;
use crate::ui::state::{
    AppState, CommandPreviewAction, CommandPreviewScreenState, ConnectionSource,
    ConnectionSourceScreenState, EngineOption, EngineSelectScreenState, ObjectExplorerScreenState,
    ProcessResultScreenState, Screen, SoftDeleteScreenState, SoftDeleteStrategy, SqlPreviewAction,
    SqlPreviewScreenState,
};
use crate::ui::use_cases::{
    config_file_connection, generator_command, generator_validation, table_preview,
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
                generator_binary_state: generator_validation::validate().state,
                last_message: generator_validation::initial_message(),
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

    pub(crate) fn ensure_generator_binary_ready(&mut self) -> bool {
        let outcome = generator_validation::validate();
        self.state.generator_binary_state = outcome.state;
        self.state.last_message = outcome.message;
        outcome.is_ready
    }

    pub(crate) fn handle_config_file_selection(&mut self) {
        use config_file_connection::ConfigFileConnectionOutcome;

        self.state.connection_source = Some(ConnectionSource::ConfigFile);

        match config_file_connection::load_or_create() {
            ConfigFileConnectionOutcome::CreatedTemplate { path, message } => {
                self.state.config_file_path = Some(path);
                self.state.last_message = message;
            }
            ConfigFileConnectionOutcome::Loaded {
                path,
                config,
                engine,
                catalog,
                message,
            } => {
                self.state.config_file_path = Some(path);
                self.state.connection_config = Some(config);
                self.state.engine = Some(engine);
                self.state.catalog = catalog;
                self.state.object_explorer_screen.selected_object = 0;
                self.state.preview = None;
                self.state.process_result = None;
                self.state.selected_db_object = None;
                self.state.screen = Screen::ObjectExplorer;
                self.state.last_message = message;
            }
            ConfigFileConnectionOutcome::Error { path, message } => {
                self.state.config_file_path = path;
                self.state.last_message = message;
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

        match table_preview::load(table_preview::LoadTablePreviewInput {
            selected: &selected,
            engine,
            catalog_mode: self.state.catalog.mode,
            catalog_tables: &self.state.catalog.tables,
            connection_config: self.state.connection_config.as_ref(),
            generator_config: self.generator_config_for_current_flow(),
            preference,
            manually_selected_field,
        }) {
            table_preview::LoadTablePreviewOutcome::Loaded(preview) => Some(preview),
            table_preview::LoadTablePreviewOutcome::Error { message } => {
                self.state.last_message = message;
                self.state.engine = Some(engine);
                None
            }
        }
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
        generator_command::execute(self.state.preview.as_ref())
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

pub(crate) fn object_type_label(object_type: DatabaseObjectType) -> &'static str {
    match object_type {
        DatabaseObjectType::Table => "table",
        DatabaseObjectType::Function => "function",
        DatabaseObjectType::StoredProcedure => "stored procedure",
    }
}
