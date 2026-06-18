use ratatui::{Frame, layout::Rect, widgets::ListItem};

use crate::ui::tui::{TuiApp, draw_menu};
use crate::ui::use_cases::soft_delete_inspection;

pub(crate) fn render(app: &TuiApp, frame: &mut Frame, area: Rect) {
    let candidates = soft_delete_inspection::manual_field_candidates(app.state.preview.as_ref());
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
