use super::super::{
    input::draw_editor,
    text::truncate,
    theme::{ACCENT, MUTED},
};
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

pub(super) fn draw_form(frame: &mut Frame, app: &App, area: Rect, spacious: bool) {
    let [label, input, _, note, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(u16::from(spacious)),
        Constraint::Length(if spacious { 3 } else { 1 }),
        Constraint::Min(0),
    ])
    .areas(area);
    frame.render_widget(
        Paragraph::new("database_path").style(Style::default().fg(MUTED)),
        label,
    );
    let editing = app.vim.input().is_some();
    let style = Style::default().add_modifier(Modifier::BOLD);
    if editing {
        draw_editor(frame, &app.editor, "› ", style, input, true);
    } else {
        let value = app
            .config
            .as_ref()
            .map_or("", |config| config.database_path.as_str());
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("› ", Style::default().fg(ACCENT)),
                Span::styled(
                    truncate(value, usize::from(input.width.saturating_sub(2))),
                    style,
                ),
            ])),
            input,
        );
    }
    frame.render_widget(
        Paragraph::new(if spacious {
            "Relative to config folder.\nSaved for the next launch."
        } else {
            "Applies on next launch."
        })
        .style(Style::default().fg(MUTED)),
        note,
    );
}
