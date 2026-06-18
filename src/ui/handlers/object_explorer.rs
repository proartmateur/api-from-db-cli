use crossterm::event::KeyCode;

use crate::domain::{DatabaseObjectType, SoftDeletePreference};
use crate::ui::state::{ConnectionSource, Screen};
use crate::ui::tui::{TuiApp, object_type_label, select_next, select_previous};

pub(crate) fn handle(app: &mut TuiApp, code: KeyCode) {
    let total = app.state.catalog.objects.len();
    match code {
        KeyCode::Esc => {
            app.state.screen = if app.state.connection_source == Some(ConnectionSource::ConfigFile)
            {
                Screen::ConnectionSource
            } else {
                Screen::EngineSelect
            };
            app.state.last_message =
                "Puedes cambiar de origen o motor sin perder control del flujo.".to_string();
        }
        KeyCode::Up => {
            select_previous(&mut app.state.object_explorer_screen.selected_object, total)
        }
        KeyCode::Down => select_next(&mut app.state.object_explorer_screen.selected_object, total),
        KeyCode::Enter => {
            if let Some(object) = app
                .state
                .catalog
                .objects
                .get(app.state.object_explorer_screen.selected_object)
                .cloned()
            {
                app.state.selected_db_object = Some(object.clone());
                app.state.screen = Screen::ObjectDetails;
                app.state.last_message = format!(
                    "Inspeccionando {} `{}`.",
                    object_type_label(object.object_type),
                    object.name
                );

                if object.object_type == DatabaseObjectType::Table {
                    app.state.preview = app.load_preview_for_selected_table(
                        SoftDeletePreference::PreferDeleteEndpoint,
                        None,
                    );
                } else {
                    app.state.preview = None;
                }
            }
        }
        _ => {}
    }
}
