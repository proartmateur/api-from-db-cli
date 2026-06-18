use ratatui::{Frame, layout::Rect, widgets::ListItem};

use crate::ui::tui::{TuiApp, draw_menu};

pub(crate) fn render(app: &TuiApp, frame: &mut Frame, area: Rect) {
    let candidates = app.manual_soft_delete_candidates();
    let items = candidates
        .iter()
        .map(|field| ListItem::new(field.clone()))
        .collect::<Vec<_>>();
    draw_menu(
        frame,
        area,
        "Columnas Datetime Disponibles",
        &items,
        app.state.soft_delete_screen.selected_manual_field,
        Some("Estas columnas ya existen y pueden mapearse como delete_at."),
    );
}
