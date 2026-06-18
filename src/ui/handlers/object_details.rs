use crossterm::event::KeyCode;

use crate::domain::DatabaseObjectType;
use crate::ui::state::{CommandPreviewAction, Screen, SoftDeleteStrategy};
use crate::ui::tui::TuiApp;
use crate::ui::use_cases::soft_delete_inspection;

pub(crate) fn handle(app: &mut TuiApp, code: KeyCode) {
    let Some(selected) = app.state.selected_db_object.clone() else {
        app.state.screen = Screen::ObjectExplorer;
        return;
    };

    match code {
        KeyCode::Esc => {
            app.state.screen = Screen::ObjectExplorer;
            app.state.last_message = "Regresaste al explorador de objetos.".to_string();
        }
        KeyCode::Enter if selected.object_type == DatabaseObjectType::Table => {
            if app.state.preview.is_none() {
                app.state.last_message =
                    "No fue posible preparar la tabla. Revisa la conexion o el schema.".to_string();
            } else if soft_delete_inspection::needs_soft_delete_decision(app.state.preview.as_ref())
            {
                app.state.screen = Screen::SoftDeleteStrategy;
                app.state.soft_delete_screen.selected_strategy =
                    SoftDeleteStrategy::CreateDeletedAt;
                app.state.last_message =
                    "No existe deleted_at compatible. Decide como quieres resolver soft delete."
                        .to_string();
            } else {
                app.state.screen = Screen::CommandPreview;
                app.state.command_preview_screen.selected_action =
                    CommandPreviewAction::ExecuteCommand;
                app.state.last_message =
                    "La tabla ya tiene suficiente metadata para construir el comando.".to_string();
            }
        }
        _ => {}
    }
}
