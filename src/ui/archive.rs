use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
};
use unicode_width::UnicodeWidthStr;

use super::{
    tasks::{draw_note, note_height},
    text::truncate,
    theme::{ACCENT, ERROR, MUTED, TEXT, priority_style},
};
use crate::app::App;

pub(super) fn draw_archive(frame: &mut Frame, app: &mut App, area: Rect) {
    let clear_count = app.archive_clear_count();
    let note_area = Rect {
        height: area.height.saturating_sub(4 + u16::from(app.error)),
        ..area
    };
    let [heading, summary, body, note, error, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(note_height(app, note_area)),
        Constraint::Length(u16::from(app.error)),
        Constraint::Length(1),
    ])
    .areas(area);
    let Some(archive) = &mut app.archive else {
        return;
    };
    frame.render_widget(
        Paragraph::new(if clear_count.is_some() {
            "Clear archive permanently?"
        } else {
            "Soft-deleted tasks"
        })
        .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        heading,
    );
    frame.render_widget(
        Paragraph::new(if let Some(count) = clear_count {
            format!("{count} tasks · all projects · no undo")
        } else {
            format!("{} archived · all projects", archive.todos.len())
        })
        .style(Style::default().fg(MUTED)),
        summary,
    );
    if archive.todos.is_empty() {
        frame.render_widget(
            Paragraph::new("No soft-deleted tasks.").style(Style::default().fg(MUTED)),
            body,
        );
    } else {
        let items = archive
            .todos
            .iter()
            .enumerate()
            .map(|(index, todo)| {
                let project = truncate(
                    archive.project_name(todo.project_id),
                    usize::from(body.width) / 4,
                );
                let title_width = usize::from(body.width).saturating_sub(project.width() + 13);
                let selected = archive.list.selected() == Some(index);
                ListItem::new(Line::from(vec![
                    Span::styled(
                        if todo.done { "✓ " } else { "□ " },
                        Style::default().fg(MUTED),
                    ),
                    Span::styled(
                        format!("{:<4} ", todo.priority.label()),
                        priority_style(todo.priority),
                    ),
                    Span::styled(
                        truncate(&todo.title, title_width),
                        Style::default().fg(TEXT).add_modifier(if selected {
                            Modifier::BOLD | Modifier::UNDERLINED
                        } else {
                            Modifier::empty()
                        }),
                    ),
                    Span::styled(format!(" · {project}"), Style::default().fg(MUTED)),
                ]))
            })
            .collect::<Vec<_>>();
        let list =
            List::new(items).highlight_symbol(Span::styled("› ", Style::default().fg(ACCENT)));
        frame.render_stateful_widget(list, body, &mut archive.list);
    }
    draw_note(frame, app, note);
    if app.error {
        frame.render_widget(
            Paragraph::new(app.status.as_str()).style(Style::default().fg(ERROR)),
            error,
        );
    }
    let hint = if clear_count.is_some() {
        "Enter clears · Esc cancels"
    } else if app.show_hints() && footer.width >= 76 {
        "j/k move · r restore tree · D clear all · Ctrl-r reload · A/Esc back · ? help"
    } else if app.show_hints() && footer.width >= 45 {
        "r restore · D clear all · A/Esc back · ? help"
    } else if app.show_hints() {
        "r restore · D clear · A/Esc back"
    } else {
        "A/Esc back · q quit"
    };
    frame.render_widget(
        Paragraph::new(hint).style(Style::default().fg(ACCENT)),
        footer,
    );
}

#[cfg(test)]
#[path = "../../tests/unit/ui/archive.rs"]
mod tests;
