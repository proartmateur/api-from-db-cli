use crossterm::event::KeyCode;

use crate::domain::DatabaseEngine;
use crate::ui::mocks;
use crate::ui::state::{ConnectionSource, Screen};
use crate::ui::tui::{TuiApp, next_connection_source, previous_connection_source};

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
                    app.state.catalog = mocks::catalog(DatabaseEngine::PostgreSql);
                    app.state.screen = Screen::EngineSelect;
                    app.state.last_message =
                        "Generador validado. Conexion mock preparada. Ahora elige el motor de base de datos."
                            .to_string();
                }
                ConnectionSource::ConfigFile => app.handle_config_file_selection(),
            }
        }
        _ => {}
    }
}
