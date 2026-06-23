use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

use crate::ui::state::ManualConnectionField;
use crate::ui::tui::{TuiApp, is_critical_helper_message, two_column_layout};

pub(crate) fn render(app: &TuiApp, frame: &mut Frame, area: Rect) {
    let [form_area, helper_area] = two_column_layout(area);

    let screen = &app.state.manual_connection_screen;
    let items = ManualConnectionField::ALL
        .iter()
        .map(|field| {
            let label = field.label();
            let raw_value = screen.value(*field);
            let value = if *field == ManualConnectionField::Password {
                "*".repeat(raw_value.len())
            } else {
                raw_value.to_string()
            };

            let marker = if screen.editing && screen.selected_field == *field {
                "✎ "
            } else {
                "  "
            };

            let line = if field.is_editable() {
                format!("{marker}{label}: {value}")
            } else {
                format!("{marker}{label}")
            };

            ListItem::new(line)
        })
        .collect::<Vec<_>>();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Conexion manual"),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");

    let mut list_state = ListState::default();
    let selected_idx = ManualConnectionField::ALL
        .iter()
        .position(|f| *f == screen.selected_field)
        .unwrap_or(0);
    list_state.select(Some(selected_idx));
    frame.render_stateful_widget(list, form_area, &mut list_state);

    let helper = build_helper(app);
    let is_alert = is_critical_helper_message(&helper);
    let helper_block = if is_alert {
        Block::default()
            .borders(Borders::ALL)
            .title("Atencion")
            .style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            )
    } else {
        Block::default().borders(Borders::ALL).title("Detalle")
    };

    let helper_widget = Paragraph::new(Text::from(vec![
        Line::from(Span::styled(
            format!("Motor: {}", app.state.engine_select_screen.selected_engine.label()),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(helper.as_str()),
    ]))
    .block(helper_block)
    .style(if is_alert {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    })
    .wrap(Wrap { trim: true });
    frame.render_widget(helper_widget, helper_area);
}

fn build_helper(app: &TuiApp) -> String {
    let screen = &app.state.manual_connection_screen;
    if screen.editing {
        format!(
            "Editando {}. Escribe el valor, Enter para confirmar, Esc para cancelar.\n\nLos campos Host, Database, Username y Password son obligatorios. Port usa el default del motor si lo dejas vacio.",
            screen.selected_field.label()
        )
    } else {
        format!(
            "{}\n\nNavega con ↑/↓, Enter para editar un campo o para Conectar.\nEsc regresa al motor. La password no se muestra en pantalla.",
            app.state.last_message
        )
    }
}