use crossterm::event::KeyCode;

use crate::ui::mocks;
use crate::ui::state::Screen;
use crate::ui::tui::{TuiApp, next_engine_option, previous_engine_option};

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
            let engine = app.state.engine_select_screen.selected_engine.to_engine();
            app.state.engine = Some(engine);
            app.state.catalog = mocks::catalog(engine);
            app.state.connection_config = None;
            app.state.object_explorer_screen.selected_object = 0;
            app.state.preview = None;
            app.state.process_result = None;
            app.state.screen = Screen::ObjectExplorer;
            app.state.last_message = format!(
                "Conexion exitosa simulada a {}. Explora los objetos disponibles.",
                engine
            );
        }
        _ => {}
    }
}
