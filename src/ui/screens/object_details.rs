use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    domain::DatabaseObjectType,
    ui::tui::{TuiApp, object_type_label, two_column_layout},
};

pub(crate) fn render(app: &TuiApp, frame: &mut Frame, area: Rect) {
    let chunks = two_column_layout(area);
    let Some(selected) = app.state.selected_db_object.as_ref() else {
        return;
    };

    let left = if selected.object_type == DatabaseObjectType::Table {
        let preview = app.state.preview.as_ref();
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
        } else {
            lines.push(Line::from(
                "No fue posible cargar la estructura de la tabla.",
            ));
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
            Line::from(app.state.last_message.clone()),
        ]
    } else {
        vec![
            Line::from("Objeto no generable en esta version."),
            Line::from("Presiona Esc para volver al explorador."),
            Line::from(""),
            Line::from(app.state.last_message.clone()),
        ]
    };
    let right = Paragraph::new(Text::from(right_lines))
        .block(Block::default().borders(Borders::ALL).title("Accion"))
        .wrap(Wrap { trim: true });
    frame.render_widget(right, chunks[1]);
}
