use crossterm::event::KeyCode;

use crate::ui::navigation::{next_engine_option, previous_engine_option};
use crate::ui::state::Screen;
use crate::ui::tui::TuiApp;

pub(crate) fn handle(app: &mut TuiApp, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.state.screen = Screen::ConnectionSource;
            app.state.last_message =
                "Regresaste a la seleccion del origen de conexion.".to_string();
        }
        KeyCode::Up => {
            app.state.engine_select_screen.selected_engine =
                previous_engine_option(app.state.engine_select_screen.selected_engine);
        }
        KeyCode::Down => {
            app.state.engine_select_screen.selected_engine =
                next_engine_option(app.state.engine_select_screen.selected_engine);
        }
        KeyCode::Enter => {
            let engine = app.state.engine_select_screen.selected_engine;
            app.state.engine = Some(engine.to_engine());
            app.state.manual_connection_screen.reset_for_engine(engine);
            app.state.screen = Screen::ManualConnection;
            app.state.last_message = format!(
                "Completa los datos para conectar a {} manualmente.",
                engine.label()
            );
        }
        _ => {}
    }
}
