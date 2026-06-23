use crossterm::event::KeyCode;

use crate::ui::state::{ManualConnectionField, Screen};
use crate::ui::tui::TuiApp;

pub(crate) fn handle(app: &mut TuiApp, code: KeyCode) {
    let screen = &mut app.state.manual_connection_screen;

    if screen.editing {
        match code {
            KeyCode::Esc => {
                screen.editing = false;
                app.state.last_message =
                    "Edicion cancelada. Los cambios no se guardaron.".to_string();
            }
            KeyCode::Enter => {
                screen.editing = false;
                app.state.last_message =
                    "Campo actualizado. Selecciona otro campo o Conectar.".to_string();
            }
            KeyCode::Backspace => {
                let field = screen.selected_field;
                screen.value_mut(field).pop();
            }
            KeyCode::Char(ch) if !ch.is_control() => {
                let field = screen.selected_field;
                screen.value_mut(field).push(ch);
            }
            _ => {}
        }
        return;
    }

    match code {
        KeyCode::Esc => {
            app.state.screen = Screen::EngineSelect;
            app.state.last_message =
                "Regresaste a la seleccion del motor de base de datos.".to_string();
        }
        KeyCode::Up => {
            screen.selected_field = screen.selected_field.previous();
        }
        KeyCode::Down => {
            screen.selected_field = screen.selected_field.next();
        }
        KeyCode::Enter => match screen.selected_field {
            ManualConnectionField::Connect => {
                app.handle_manual_connection_submit();
            }
            field if field.is_editable() => {
                screen.editing = true;
                app.state.last_message = format!(
                    "Editando {}. Enter para confirmar, Esc para cancelar.",
                    field.label()
                );
            }
            _ => {}
        },
        _ => {}
    }
}