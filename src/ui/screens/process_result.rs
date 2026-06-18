use ratatui::{
    Frame,
    layout::Rect,
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::ui::tui::TuiApp;

pub(crate) fn render(app: &TuiApp, frame: &mut Frame, area: Rect) {
    let lines = app.build_process_result_lines();
    let widget = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Resultado del Generador"),
        )
        .scroll((app.state.process_result_screen.scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(Clear, area);
    frame.render_widget(widget, area);
}
