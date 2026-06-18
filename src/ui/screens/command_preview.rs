use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Text},
    widgets::{Block, Borders, ListItem, Paragraph, Wrap},
};

use crate::ui::tui::{
    CommandPreviewAction, TuiApp, render_selectable_list, selected_index, two_column_layout,
};

pub(crate) fn render(app: &TuiApp, frame: &mut Frame, area: Rect) {
    let chunks = two_column_layout(area);
    let preview = app.state.preview.as_ref();
    let generator = app.generator_config_for_current_flow();
    let command = preview
        .map(|item| item.generated_command.raw_command.clone())
        .unwrap_or_else(|| "No hay comando generado.".to_string());
    let flags = if generator.flags.is_empty() {
        "(sin flags)".to_string()
    } else {
        generator.flags.join(" ")
    };

    let lines = vec![
        Line::from("Configuracion activa del generador:"),
        Line::from(format!("cmd: {}", generator.cmd)),
        Line::from(format!("flags: {}", flags)),
        Line::from(""),
        Line::from("Comando final:"),
        Line::from(command),
        Line::from(""),
        Line::from(
            preview
                .map(|item| format!("Soft delete: {}", item.soft_delete.summary()))
                .unwrap_or_else(|| "Soft delete no disponible".to_string()),
        ),
    ];
    let command_widget = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Preview de Comando"),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(command_widget, chunks[0]);

    let actions = CommandPreviewAction::ALL
        .iter()
        .map(|action| ListItem::new(action.label()))
        .collect::<Vec<_>>();
    render_selectable_list(
        frame,
        chunks[1],
        "Acciones",
        &actions,
        selected_index(
            CommandPreviewAction::ALL,
            app.state.command_preview_screen.selected_action,
        ),
    );
}
