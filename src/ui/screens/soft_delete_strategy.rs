use ratatui::{Frame, layout::Rect, widgets::ListItem};

use crate::ui::tui::{SoftDeleteStrategy, TuiApp, draw_menu, selected_index};

pub(crate) fn render(app: &TuiApp, frame: &mut Frame, area: Rect) {
    let items = SoftDeleteStrategy::ALL
        .iter()
        .map(|strategy| ListItem::new(strategy.label()))
        .collect::<Vec<_>>();

    draw_menu(
        frame,
        area,
        "Resolver Soft Delete",
        &items,
        selected_index(
            SoftDeleteStrategy::ALL,
            app.state.soft_delete_screen.selected_strategy,
        ),
        Some(app.state.last_message.as_str()),
    );
}
