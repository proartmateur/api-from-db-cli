use crossterm::event::KeyCode;

use crate::ui::navigation::{select_next, select_previous};
use crate::ui::state::{CommandPreviewAction, Screen, SqlPreviewAction};
use crate::ui::tui::TuiApp;
use crate::ui::use_cases::soft_delete_inspection;
use crate::ui::use_cases::soft_delete_resolution::{
    self, ResolveSoftDeleteOutcome, ResolveSoftDeleteWithFieldInput,
};

pub(crate) fn handle(app: &mut TuiApp, code: KeyCode) {
    let candidates = soft_delete_inspection::manual_field_candidates(app.state.preview.as_ref());
    let total = candidates.len();
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
            if let Some(field) = candidates
                .get(app.state.soft_delete_screen.selected_manual_field)
                .cloned()
            {
                match soft_delete_resolution::resolve_with_existing_field(
                    ResolveSoftDeleteWithFieldInput {
                        field: field.as_str(),
                        selected: app
                            .state
                            .selected_db_object
                            .as_ref()
                            .expect("selected object must exist"),
                        engine: app.state.engine.expect("engine must exist"),
                        catalog_mode: app.state.catalog.mode,
                        catalog_tables: &app.state.catalog.tables,
                        connection_config: app.state.connection_config.as_ref(),
                        generator_config: app.generator_config_for_current_flow(),
                    },
                ) {
                    ResolveSoftDeleteOutcome::ShowCommandPreview { preview, message } => {
                        app.state.preview = Some(preview);
                        app.state.screen = Screen::CommandPreview;
                        app.state.command_preview_screen.selected_action =
                            CommandPreviewAction::ExecuteCommand;
                        app.state.last_message = message;
                    }
                    ResolveSoftDeleteOutcome::Error { message } => {
                        app.state.last_message = message;
                    }
                    ResolveSoftDeleteOutcome::ShowSqlPreview { preview, message } => {
                        app.state.preview = Some(preview);
                        app.state.screen = Screen::SqlPreview;
                        app.state.sql_preview_screen.selected_action =
                            SqlPreviewAction::ExecuteAndContinue;
                        app.state.last_message = message;
                    }
                    ResolveSoftDeleteOutcome::RequestManualField { message } => {
                        app.state.last_message = message;
                    }
                }
            }
        }
        _ => {}
    }
}
