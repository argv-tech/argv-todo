use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use unicode_width::UnicodeWidthStr;

use super::super::{
    text::truncate,
    theme::{ACCENT, MUTED},
};
use crate::app::App;

pub(in crate::ui) fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    if area.is_empty() {
        return;
    }
    let active = app.active_project_index();
    let width = usize::from(area.width);
    let name_width = width.saturating_sub(6).min(22);
    let labels: Vec<_> = app
        .project_tabs()
        .enumerate()
        .map(|(index, (_, name))| {
            let name = truncate(name, name_width);
            if index == active {
                format!("[{name}]")
            } else {
                format!(" {name} ")
            }
        })
        .collect();
    // Keep the active tab visible, then fill remaining columns with its neighbors.
    let mut start = active;
    let mut end = active + 1;
    let mut used = labels[active].width();
    while start > 0 && used + labels[start - 1].width() + 4 <= width {
        start -= 1;
        used += labels[start].width();
    }
    while end < labels.len() && used + labels[end].width() + 4 <= width {
        used += labels[end].width();
        end += 1;
    }
    let mut spans = Vec::new();
    if start > 0 {
        spans.push(Span::styled("‹ ", Style::default().fg(MUTED)));
    }
    for (index, label) in labels.into_iter().enumerate().take(end).skip(start) {
        let style = if index == active {
            Style::default()
                .fg(ACCENT)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(MUTED)
        };
        spans.push(Span::styled(label, style));
    }
    if end < app.project_tabs().len() {
        spans.push(Span::styled(" ›", Style::default().fg(MUTED)));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
