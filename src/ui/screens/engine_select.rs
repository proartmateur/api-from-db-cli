use ratatui::{Frame, layout::Rect, widgets::ListItem};

use crate::ui::state::EngineOption;
use crate::ui::tui::{TuiApp, draw_menu, selected_index};

pub(crate) fn render(app: &TuiApp, frame: &mut Frame, area: Rect) {
    let items = EngineOption::ALL
        .iter()
        .map(|option| ListItem::new(option.label()))
        .collect::<Vec<_>>();
    draw_menu(
        frame,
        area,
        "Motor",
        &items,
        selected_index(
            EngineOption::ALL,
            app.state.engine_select_screen.selected_engine,
        ),
        Some(app.state.last_message.as_str()),
    );
}
