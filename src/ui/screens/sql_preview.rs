use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, ListItem, Paragraph, Wrap},
};

use crate::ui::tui::{
    SqlPreviewAction, TuiApp, render_selectable_list, selected_index, two_column_layout,
};

pub(crate) fn render(app: &TuiApp, frame: &mut Frame, area: Rect) {
    let chunks = two_column_layout(area);
    let preview = app.state.preview.as_ref();
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

    let actions = SqlPreviewAction::ALL
        .iter()
        .map(|action| ListItem::new(action.label()))
        .collect::<Vec<_>>();
    render_selectable_list(
        frame,
        chunks[1],
        "Acciones",
        &actions,
        selected_index(
            SqlPreviewAction::ALL,
            app.state.sql_preview_screen.selected_action,
        ),
    );
}
