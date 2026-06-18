use crossterm::event::KeyCode;

use crate::domain::SoftDeletePreference;
use crate::ui::state::{CommandPreviewAction, Screen, SoftDeleteStrategy, SqlPreviewAction};
use crate::ui::tui::{TuiApp, next_soft_delete_strategy, previous_soft_delete_strategy};

pub(crate) fn handle(app: &mut TuiApp, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.state.screen = Screen::ObjectDetails;
            app.state.last_message =
                "Puedes revisar de nuevo la tabla antes de decidir.".to_string();
        }
        KeyCode::Up => {
            app.state.soft_delete_screen.selected_strategy =
                previous_soft_delete_strategy(app.state.soft_delete_screen.selected_strategy);
        }
        KeyCode::Down => {
            app.state.soft_delete_screen.selected_strategy =
                next_soft_delete_strategy(app.state.soft_delete_screen.selected_strategy);
        }
        KeyCode::Enter => match app.state.soft_delete_screen.selected_strategy {
            SoftDeleteStrategy::CreateDeletedAt => {
                app.state.preview = app.load_preview_for_selected_table(
                    SoftDeletePreference::PreferDeleteEndpoint,
                    None,
                );
                app.state.screen = Screen::SqlPreview;
                app.state.sql_preview_screen.selected_action = SqlPreviewAction::ExecuteAndContinue;
                app.state.last_message =
                    "Se genero el ALTER TABLE para revisar antes de continuar.".to_string();
            }
            SoftDeleteStrategy::UseExistingField => {
                app.state.soft_delete_screen.selected_manual_field = 0;
                app.state.screen = Screen::ManualSoftDeleteField;
                app.state.last_message =
                    "Selecciona una columna datetime existente para soft delete.".to_string();
            }
            SoftDeleteStrategy::ContinueWithoutDelete => {
                app.state.preview = app.load_preview_for_selected_table(
                    SoftDeletePreference::SkipDeleteEndpoint,
                    None,
                );
                app.state.screen = Screen::CommandPreview;
                app.state.command_preview_screen.selected_action =
                    CommandPreviewAction::ExecuteCommand;
                app.state.last_message = "Seguimos sin endpoint delete para esta API.".to_string();
            }
        },
        _ => {}
    }
}
