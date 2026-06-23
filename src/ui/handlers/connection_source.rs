use crossterm::event::KeyCode;

use crate::ui::navigation::{next_connection_source, previous_connection_source};
use crate::ui::state::{ConnectionSource, Screen};
use crate::ui::tui::TuiApp;

pub(crate) fn handle(app: &mut TuiApp, code: KeyCode) {
    match code {
        KeyCode::Up => {
            app.state.connection_source_screen.selected_source =
                previous_connection_source(app.state.connection_source_screen.selected_source);
        }
        KeyCode::Down => {
            app.state.connection_source_screen.selected_source =
                next_connection_source(app.state.connection_source_screen.selected_source);
        }
        KeyCode::Enter => {
            if !app.ensure_generator_binary_ready() {
                return;
            }

            match app.state.connection_source_screen.selected_source {
                ConnectionSource::Manual => {
                    app.state.connection_source = Some(ConnectionSource::Manual);
                    app.state.connection_config = None;
                    app.state.manual_connection_screen.reset_for_engine(
                        app.state.engine_select_screen.selected_engine,
                    );
                    app.state.screen = Screen::EngineSelect;
                    app.state.last_message =
                        "Generador validado. Elige el motor para la conexion manual.".to_string();
                }
                ConnectionSource::ConfigFile => app.handle_config_file_selection(),
            }
        }
        _ => {}
    }
}
