use ratatui::{Frame, layout::Rect, widgets::ListItem};

use crate::ui::tui::{ConnectionSource, TuiApp, draw_menu, selected_index};

pub(crate) fn render(app: &TuiApp, frame: &mut Frame, area: Rect) {
    let items = ConnectionSource::ALL
        .iter()
        .map(|source| ListItem::new(source.label()))
        .collect::<Vec<_>>();

    let helper_message = match app.state.config_file_path.as_deref() {
        Some(path) => format!(
            "{}\n\nRuta actual del archivo: {}",
            app.state.last_message, path
        ),
        None => app.state.last_message.clone(),
    };

    draw_menu(
        frame,
        area,
        "Origen de Conexion",
        &items,
        selected_index(
            ConnectionSource::ALL,
            app.state.connection_source_screen.selected_source,
        ),
        Some(helper_message.as_str()),
    );
}
