use crossterm::event::KeyCode;

use crate::ui::navigation::{next_sql_preview_action, previous_sql_preview_action};
use crate::ui::state::{CommandPreviewAction, Screen, SqlPreviewAction};
use crate::ui::tui::TuiApp;

pub(crate) fn handle(app: &mut TuiApp, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.state.screen = Screen::SoftDeleteStrategy;
            app.state.last_message =
                "Puedes elegir otra estrategia antes de ejecutar nada.".to_string();
        }
        KeyCode::Up => {
            app.state.sql_preview_screen.selected_action =
                previous_sql_preview_action(app.state.sql_preview_screen.selected_action);
        }
        KeyCode::Down => {
            app.state.sql_preview_screen.selected_action =
                next_sql_preview_action(app.state.sql_preview_screen.selected_action);
        }
        KeyCode::Enter => match app.state.sql_preview_screen.selected_action {
            SqlPreviewAction::ExecuteAndContinue => {
                app.state.screen = Screen::CommandPreview;
                app.state.command_preview_screen.selected_action =
                    CommandPreviewAction::ExecuteCommand;
                app.state.last_message =
                    "ALTER TABLE confirmado para este flujo. La metadata se considera refrescada."
                        .to_string();
            }
            SqlPreviewAction::CopyAndContinue => {
                app.state.screen = Screen::CommandPreview;
                app.state.command_preview_screen.selected_action =
                    CommandPreviewAction::ExecuteCommand;
                app.state.last_message =
                    "SQL marcado como copiado. Puedes ejecutarlo aparte y seguir al comando."
                        .to_string();
            }
            SqlPreviewAction::CopyAndStay => {
                app.state.last_message =
                    "SQL marcado como copiado. Puedes ejecutarlo aparte cuando quieras."
                        .to_string();
            }
            SqlPreviewAction::Cancel => {
                app.state.screen = Screen::SoftDeleteStrategy;
                app.state.last_message = "Operacion cancelada. La base sigue intacta.".to_string();
            }
        },
        _ => {}
    }
}
