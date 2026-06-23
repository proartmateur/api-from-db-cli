use crossterm::event::KeyCode;

use crate::adapters::clipboard::ArboardClipboard;
use crate::core::ports::Clipboard;
use crate::domain::ProcessResult;
use crate::ui::mocks::process;
use crate::ui::navigation::{next_command_preview_action, previous_command_preview_action};
use crate::ui::state::{CommandPreviewAction, Screen};
use crate::ui::tui::TuiApp;

pub(crate) fn handle(app: &mut TuiApp, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.state.screen = if app
                .state
                .preview
                .as_ref()
                .and_then(|preview| preview.generated_sql.as_ref())
                .is_some()
            {
                Screen::SqlPreview
            } else {
                Screen::ObjectDetails
            };
            app.state.last_message =
                "Puedes revisar el paso anterior antes de ejecutar.".to_string();
        }
        KeyCode::Up => {
            app.state.command_preview_screen.selected_action =
                previous_command_preview_action(app.state.command_preview_screen.selected_action);
        }
        KeyCode::Down => {
            app.state.command_preview_screen.selected_action =
                next_command_preview_action(app.state.command_preview_screen.selected_action);
        }
        KeyCode::Enter => match app.state.command_preview_screen.selected_action {
            CommandPreviewAction::ExecuteCommand => match app.run_generated_command() {
                Ok(result) => {
                    app.state.process_result = Some(result);
                    app.state.process_result_screen.scroll = 0;
                    app.state.screen = Screen::ProcessResult;
                    app.state.last_message =
                        "Ejecucion completada. Ya puedes revisar stdout y stderr.".to_string();
                }
                Err(error) => {
                    app.state.process_result = Some(ProcessResult {
                        exit_code: -1,
                        stdout: String::new(),
                        stderr: error,
                        success: false,
                    });
                    app.state.process_result_screen.scroll = 0;
                    app.state.screen = Screen::ProcessResult;
                    app.state.last_message =
                        "La ejecucion del comando fallo antes de completar el proceso.".to_string();
                }
            },
            CommandPreviewAction::CopyAndMarkExternalExecution => {
                copy_command_to_clipboard(app);
                app.state.process_result =
                    Some(process::external_process_result(app.state.preview.as_ref()));
                app.state.process_result_screen.scroll = 0;
                app.state.screen = Screen::ProcessResult;
                app.state.last_message =
                    "Comando copiado al portapapeles para ejecucion externa.".to_string();
            }
            CommandPreviewAction::CopyAndStay => {
                copy_command_to_clipboard(app);
                app.state.last_message =
                    "Comando copiado al portapapeles.".to_string();
            }
            CommandPreviewAction::Cancel => {
                app.state.screen = Screen::ObjectExplorer;
                app.state.last_message =
                    "Ejecucion cancelada. Regresaste al explorador de objetos.".to_string();
            }
        },
        _ => {}
    }
}

fn copy_command_to_clipboard(app: &mut TuiApp) {
    let command = app
        .state
        .preview
        .as_ref()
        .map(|preview| preview.generated_command.raw_command.clone())
        .unwrap_or_else(|| "No hay comando generado.".to_string());

    match ArboardClipboard.copy(&command) {
        Ok(()) => {}
        Err(error) => {
            app.state.last_message = error.to_string();
        }
    }
}
