use crate::vim_motion::Editor;
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
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
    let (text, column) = editor.viewport(area.width.saturating_sub(prefix_width));
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(label, Style::default().fg(ACCENT)),
            Span::styled(text, style),
        ])),
        area,
    );
    if focused && prefix_width < area.width {
        frame.set_cursor_position((area.x + prefix_width + column, area.y));
    }
}
