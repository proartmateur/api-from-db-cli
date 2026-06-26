use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, ListItem, Paragraph, Wrap},
};

use crate::domain::DatabaseObjectType;
use crate::ui::state::{CatalogMode, ConnectionSource};
use crate::ui::tui::{TuiApp, object_type_label, render_selectable_list, two_column_layout};

pub(crate) fn render(app: &TuiApp, frame: &mut Frame, area: Rect) {
    let chunks = two_column_layout(area);

    let screen = &app.state.object_explorer_screen;
    let filter = &screen.filter;

    // Build filtered items
    let filtered_objects: Vec<_> = app
        .state
        .catalog
        .objects
        .iter()
        .filter(|o| {
            filter.is_empty() || o.name.to_lowercase().contains(&filter.to_lowercase())
        })
        .collect();

    let items = filtered_objects
        .iter()
        .map(|object| {
            let schema = object.schema.as_deref().unwrap_or("<sin schema>");
            let type_color = match object.object_type {
                DatabaseObjectType::Table => Color::Green,
                DatabaseObjectType::Function => Color::Cyan,
                DatabaseObjectType::StoredProcedure => Color::Magenta,
            };
            let type_span = Span::styled(
                object_type_label(object.object_type),
                Style::default().fg(type_color).add_modifier(Modifier::BOLD),
            );
            ListItem::new(Line::from(vec![
                Span::raw(format!("[{schema}] ")),
                type_span,
                Span::raw(format!(" {}", object.name)),
            ]))
        })
        .collect::<Vec<_>>();

    let tables = app.state.catalog.objects.iter().filter(|o| o.object_type == DatabaseObjectType::Table).count();
    let functions = app.state.catalog.objects.iter().filter(|o| o.object_type == DatabaseObjectType::Function).count();
    let procs = app.state.catalog.objects.iter().filter(|o| o.object_type == DatabaseObjectType::StoredProcedure).count();

    let title = if filter.is_empty() {
        format!("Objetos Disponibles ({tables}T,{functions}F,{procs}SP)")
    } else {
        format!("Objetos Disponibles ({}/{} encontrados)", filtered_objects.len(), tables + functions + procs)
    };

    // Split left column: list + optional search input at bottom
    let left_chunks = if screen.searching {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(chunks[0])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1)])
            .split(chunks[0])
    };

    render_selectable_list(
        frame,
        left_chunks[0],
        &title,
        &items,
        screen.selected_object,
    );

    if screen.searching {
        let search_input = Paragraph::new(Line::from(vec![
            Span::styled("Buscar: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(filter.as_str(), Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]))
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(search_input, left_chunks[1]);
    }

    let right_text = vec![
        Line::from(match app.state.catalog.mode {
            CatalogMode::Mock => "Conexion mock activa.",
            CatalogMode::Real => "Conexion real activa.",
        }),
        Line::from(format!(
            "Origen: {}",
            app.state
                .connection_source
                .map(ConnectionSource::label)
                .unwrap_or("Pendiente")
        )),
        Line::from(format!(
            "Motor: {}",
            app.state
                .engine
                .map(|engine| engine.to_string())
                .unwrap_or_else(|| "pendiente".to_string())
        )),
        Line::from(""),
        Line::from(if screen.searching {
            "Escribe para filtrar. Esc limpia el filtro. Esc de nuevo para volver.".to_string()
        } else {
            "Presiona Space para buscar por nombre.".to_string()
        }),
        Line::from(app.state.last_message.clone()),
    ];
    let details = Paragraph::new(Text::from(right_text))
        .block(Block::default().borders(Borders::ALL).title("Contexto"))
        .wrap(Wrap { trim: true });
    frame.render_widget(details, chunks[1]);
}
