use crossterm::event::KeyCode;

use crate::domain::{DatabaseObjectType, SoftDeletePreference};
use crate::ui::navigation::{select_next, select_previous};
use crate::ui::state::{ConnectionSource, Screen};
use crate::ui::tui::{TuiApp, object_type_label};

pub(crate) fn handle(app: &mut TuiApp, code: KeyCode) {
    let screen = &mut app.state.object_explorer_screen;

    // Build filtered list indices once so navigation stays consistent
    let filtered: Vec<usize> = app
        .state
        .catalog
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            screen.filter.is_empty()
                || o.name.to_lowercase().contains(&screen.filter.to_lowercase())
        })
        .map(|(i, _)| i)
        .collect();

    if screen.searching {
        match code {
            KeyCode::Esc => {
                if screen.filter.is_empty() {
                    screen.searching = false;
                } else {
                    screen.filter.clear();
                    screen.selected_object = 0;
                }
            }
            KeyCode::Backspace => {
                screen.filter.pop();
                screen.selected_object = 0;
            }
            KeyCode::Char(ch) if !ch.is_control() => {
                screen.filter.push(ch);
                screen.selected_object = 0;
            }
            KeyCode::Up => select_previous(&mut screen.selected_object, filtered.len()),
            KeyCode::Down => select_next(&mut screen.selected_object, filtered.len()),
            KeyCode::Enter => {
                if let Some(&real_idx) = filtered.get(screen.selected_object) {
                    navigate_to_object(app, real_idx);
                }
            }
            _ => {}
        }
        return;
    }

    // Normal (non-search) mode
    match code {
        KeyCode::Char(' ') => {
            app.state.object_explorer_screen.searching = true;
            app.state.object_explorer_screen.selected_object = 0;
        }
        KeyCode::Esc => {
            app.state.screen = match app.state.connection_source {
                Some(ConnectionSource::ConfigFile) => Screen::ConnectionSource,
                Some(ConnectionSource::Manual) => Screen::ManualConnection,
                None => Screen::EngineSelect,
            };
            app.state.last_message =
                "Puedes cambiar de origen o motor sin perder control del flujo.".to_string();
        }
        KeyCode::Up => {
            select_previous(&mut app.state.object_explorer_screen.selected_object, filtered.len())
        }
        KeyCode::Down => {
            select_next(&mut app.state.object_explorer_screen.selected_object, filtered.len())
        }
        KeyCode::Enter => {
            if let Some(&real_idx) = filtered.get(app.state.object_explorer_screen.selected_object) {
                navigate_to_object(app, real_idx);
            }
        }
        _ => {}
    }
}

fn navigate_to_object(app: &mut TuiApp, real_idx: usize) {
    if let Some(object) = app.state.catalog.objects.get(real_idx).cloned() {
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
