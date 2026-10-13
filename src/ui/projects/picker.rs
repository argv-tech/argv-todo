use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use super::super::{
    text::truncate,
    theme::{ACCENT, BG, ERROR, MUTED, TEXT},
};
use crate::app::App;

pub(in crate::ui) fn draw_picker(frame: &mut Frame, app: &mut App) {
    let Some((task_id, _)) = app.project_picker() else {
        return;
    };
    let area = frame.area();
    let width = area.width.saturating_sub(4).min(56);
    let height = area.height.saturating_sub(2).min(
        app.project_tabs()
            .len()
            .saturating_add(6)
            .min(usize::from(u16::MAX)) as u16,
    );
    let popup = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Move task tree to project ")
        .border_style(Style::default().fg(ACCENT))
        .style(Style::default().bg(BG).fg(TEXT));
    let inner = block.inner(popup);
    frame.render_widget(Clear, popup);
    frame.render_widget(block, popup);
    let [heading, _, list, error, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(u16::from(app.error)),
        Constraint::Length(1),
    ])
    .areas(inner);
    let title = app
        .todos
        .iter()
        .find(|task| task.id == task_id)
        .map_or("Task", |task| task.title.as_str());
    frame.render_widget(
        Paragraph::new(truncate(title, usize::from(heading.width))),
        heading,
    );
    let items: Vec<_> = app
        .project_tabs()
        .map(|(_, name)| {
            ListItem::new(truncate(name, usize::from(list.width.saturating_sub(2))).into_owned())
        })
        .collect();
    if let Some(state) = app.project_picker_list_mut() {
        frame.render_stateful_widget(
            List::new(items)
                .highlight_symbol("› ")
                .highlight_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            list,
            state,
        );
    }
    if app.error {
        frame.render_widget(
            Paragraph::new(app.status.as_str()).style(Style::default().fg(ERROR)),
            error,
        );
    }
    let hint = if footer.width >= 36 {
        "j/k choose · Enter move · Esc cancel"
    } else {
        "j/k · Enter move · Esc"
    };
    frame.render_widget(
        Paragraph::new(hint).style(Style::default().fg(MUTED)),
        footer,
    );
}
