use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, ListItem, Paragraph, Wrap},
};

use crate::domain::DatabaseObjectType;
use crate::ui::state::{CatalogMode, ConnectionSource};
use crate::ui::tui::{TuiApp, object_type_label, render_selectable_list, two_column_layout};

pub(crate) fn render(app: &TuiApp, frame: &mut Frame, area: Rect) {
    let chunks = two_column_layout(area);
    let items = app
        .state
        .catalog
        .objects
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
                Style::default()
                    .fg(type_color)
                    .add_modifier(Modifier::BOLD),
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
    let title = format!("Objetos Disponibles ({tables}T,{functions}F,{procs}SP)");

    render_selectable_list(
        frame,
        chunks[0],
        &title,
        &items,
        app.state.object_explorer_screen.selected_object,
    );

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
        Line::from(match app.state.catalog.mode {
            CatalogMode::Mock => {
                "Incluye tablas, funciones o stored procedures simulados.".to_string()
            }
            CatalogMode::Real => {
                "Los objetos vienen desde SQL Server usando el archivo de configuracion."
                    .to_string()
            }
        }),
        Line::from(app.state.last_message.clone()),
    ];
    let details = Paragraph::new(Text::from(right_text))
        .block(Block::default().borders(Borders::ALL).title("Contexto"))
        .wrap(Wrap { trim: true });
    frame.render_widget(details, chunks[1]);
}
