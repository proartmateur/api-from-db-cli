use crossterm::event::KeyCode;

use crate::domain::SoftDeletePreference;
use crate::ui::navigation::{select_next, select_previous};
use crate::ui::state::{CommandPreviewAction, Screen};
use crate::ui::tui::TuiApp;

pub(crate) fn handle(app: &mut TuiApp, code: KeyCode) {
    let total = app.manual_soft_delete_candidates().len();
    match code {
        KeyCode::Esc => {
            app.state.screen = Screen::SoftDeleteStrategy;
            app.state.last_message =
                "Regresaste a las opciones para resolver soft delete.".to_string();
        }
        KeyCode::Up => select_previous(
            &mut app.state.soft_delete_screen.selected_manual_field,
            total,
        ),
        KeyCode::Down => select_next(
            &mut app.state.soft_delete_screen.selected_manual_field,
            total,
        ),
        KeyCode::Enter => {
            if let Some(field) = app
                .manual_soft_delete_candidates()
                .get(app.state.soft_delete_screen.selected_manual_field)
                .cloned()
            {
                app.state.preview = app.load_preview_for_selected_table(
                    SoftDeletePreference::PreferDeleteEndpoint,
                    Some(field.as_str()),
                );
                app.state.screen = Screen::CommandPreview;
                app.state.command_preview_screen.selected_action =
                    CommandPreviewAction::ExecuteCommand;
                app.state.last_message =
                    format!("Usaremos `{}` como columna de soft delete.", field);
            }
        }
        _ => {}
    }
}
