use crossterm::event::KeyCode;

use crate::ui::navigation::{next_soft_delete_strategy, previous_soft_delete_strategy};
use crate::ui::state::{CommandPreviewAction, Screen, SqlPreviewAction};
use crate::ui::tui::TuiApp;
use crate::ui::use_cases::soft_delete_resolution::{
    self, ResolveSoftDeleteInput, ResolveSoftDeleteOutcome,
};

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
        KeyCode::Enter => match soft_delete_resolution::resolve(ResolveSoftDeleteInput {
            strategy: app.state.soft_delete_screen.selected_strategy,
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
        }) {
            ResolveSoftDeleteOutcome::ShowSqlPreview { preview, message } => {
                app.state.preview = Some(preview);
                app.state.screen = Screen::SqlPreview;
                app.state.sql_preview_screen.selected_action = SqlPreviewAction::ExecuteAndContinue;
                app.state.last_message = message;
            }
            ResolveSoftDeleteOutcome::ShowCommandPreview { preview, message } => {
                app.state.preview = Some(preview);
                app.state.screen = Screen::CommandPreview;
                app.state.command_preview_screen.selected_action =
                    CommandPreviewAction::ExecuteCommand;
                app.state.last_message = message;
            }
            ResolveSoftDeleteOutcome::RequestManualField { message } => {
                app.state.soft_delete_screen.selected_manual_field = 0;
                app.state.screen = Screen::ManualSoftDeleteField;
                app.state.last_message = message;
            }
            ResolveSoftDeleteOutcome::Error { message } => {
                app.state.last_message = message;
            }
        },
        _ => {}
    }
}
