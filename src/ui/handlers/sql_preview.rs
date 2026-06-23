use crossterm::event::KeyCode;

use crate::adapters::clipboard::ArboardClipboard;
use crate::core::ports::Clipboard;
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
                copy_sql_to_clipboard(app);
                app.state.screen = Screen::CommandPreview;
                app.state.command_preview_screen.selected_action =
                    CommandPreviewAction::ExecuteCommand;
                app.state.last_message =
                    "SQL copiado al portapapeles. Puedes ejecutarlo aparte y seguir al comando."
                        .to_string();
            }
            SqlPreviewAction::CopyAndStay => {
                copy_sql_to_clipboard(app);
                app.state.last_message =
                    "SQL copiado al portapapeles. Puedes ejecutarlo aparte cuando quieras."
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

fn copy_sql_to_clipboard(app: &mut TuiApp) {
    let sql = app
        .state
        .preview
        .as_ref()
        .and_then(|preview| preview.generated_sql.as_ref())
        .map(|generated| generated.sql.clone())
        .unwrap_or_else(|| "No hay SQL generado.".to_string());

    match ArboardClipboard.copy(&sql) {
        Ok(()) => {}
        Err(error) => {
            app.state.last_message = error.to_string();
        }
    }
}
