use crate::vim_motion::Editor;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use unicode_width::UnicodeWidthStr;

use super::theme::ACCENT;

/// Search and configuration share the same scrolling and cursor placement.
pub(super) fn draw_editor(
    frame: &mut Frame,
    editor: &Editor,
    label: &str,
    style: Style,
    area: Rect,
    focused: bool,
) {
    if area.is_empty() {
        return;
    }
    let prefix_width = label.width().min(usize::from(area.width)) as u16;
    let (mut spans, column) = editor_spans(editor, area.width.saturating_sub(prefix_width), style);
    spans.insert(0, Span::styled(label, Style::default().fg(ACCENT)));
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
    if focused && prefix_width < area.width {
        frame.set_cursor_position((area.x + prefix_width + column, area.y));
    }
}

/// Shared selection styling for inline titles, search, and configuration.
pub(super) fn editor_spans(editor: &Editor, width: u16, style: Style) -> (Vec<Span<'_>>, u16) {
    let (text, column) = editor.viewport(width);
    let Some(selection) = editor.selection() else {
        return (vec![Span::styled(text, style)], column);
    };
    let offset = editor.text().len() - text.len();
    let start = selection.start.saturating_sub(offset).min(text.len());
    let end = selection.end.saturating_sub(offset).min(text.len());
    (
        vec![
            Span::styled(&text[..start], style),
            Span::styled(&text[start..end], style.add_modifier(Modifier::REVERSED)),
            Span::styled(&text[end..], style),
        ],
        column,
    )
}
