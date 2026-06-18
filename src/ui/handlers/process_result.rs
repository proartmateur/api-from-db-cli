use crossterm::event::KeyCode;

use crate::ui::state::Screen;
use crate::ui::tui::TuiApp;

pub(crate) fn handle(app: &mut TuiApp, code: KeyCode) {
    let total_lines = app.process_result_line_count() as u16;
    match code {
        KeyCode::Up => {
            app.state.process_result_screen.scroll =
                app.state.process_result_screen.scroll.saturating_sub(1);
        }
        KeyCode::Down => {
            app.state.process_result_screen.scroll = app
                .state
                .process_result_screen
                .scroll
                .saturating_add(1)
                .min(total_lines.saturating_sub(1));
        }
        KeyCode::PageUp => {
            app.state.process_result_screen.scroll =
                app.state.process_result_screen.scroll.saturating_sub(10);
        }
        KeyCode::PageDown => {
            app.state.process_result_screen.scroll = app
                .state
                .process_result_screen
                .scroll
                .saturating_add(10)
                .min(total_lines.saturating_sub(1));
        }
        KeyCode::Home => {
            app.state.process_result_screen.scroll = 0;
        }
        KeyCode::End => {
            app.state.process_result_screen.scroll = total_lines.saturating_sub(1);
        }
        KeyCode::Esc | KeyCode::Enter => {
            app.state.screen = Screen::ObjectExplorer;
            app.state.last_message =
                "El flujo termino. Puedes probar otra tabla o cambiar de motor.".to_string();
        }
        _ => {}
    }
}
