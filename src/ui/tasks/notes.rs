use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};

use super::super::{
    input::draw_multiline_editor,
    text::{truncate, wrap},
    theme::{ACCENT, MUTED, TEXT},
};
use crate::{app::App, config::TaskView, vim_motion::InputTarget};
use unicode_width::UnicodeWidthStr;

pub(in crate::ui) fn note_height(app: &App, area: Rect) -> u16 {
    if area.is_empty() {
        return 0;
    }
    let tree_rows =
        if app.archive.is_none() && app.task_view() == TaskView::Split && area.width < 70 {
            5
        } else {
            3
        };
    if app.vim.input() == Some(InputTarget::Note) {
        let maximum = area
            .height
            .saturating_sub(tree_rows)
            .clamp(2, 7)
            .min(area.height);
        return (app.editor.text().split('\n').count() + 1)
            .max(2)
            .min(usize::from(maximum)) as u16;
    }
    let maximum = area.height.saturating_sub(tree_rows).min(5);
    // Keep both stacked task lists and the active search visible on short terminals.
    if maximum < 2 {
        return 0;
    }
    let (text, _) = note_text(app);
    let lines = note_lines(text, usize::from(area.width.saturating_sub(2))).len();
    lines.saturating_add(1).clamp(2, usize::from(maximum)) as u16
}

pub(in crate::ui) fn draw_note(frame: &mut Frame, app: &App, area: Rect) {
    if area.is_empty() {
        return;
    }
    let editing = app.vim.input() == Some(InputTarget::Note);
    let heading = if editing { " › Note " } else { " Note " };
    let hint = if !app.show_hints() {
        ""
    } else if editing {
        " Ctrl-s save "
    } else if app.archive.is_none() && app.selected_todo().is_some() && !adding_task(app) {
        " n edit "
    } else {
        ""
    };
    let title_width = usize::from(area.width).saturating_sub(heading.width() + hint.width() + 3);
    let mut title = vec![Span::styled(
        heading,
        Style::default()
            .fg(if editing { ACCENT } else { TEXT })
            .add_modifier(Modifier::BOLD),
    )];
    if !adding_task(app)
        && let Some(todo) = app.note_todo()
        && title_width > 0
    {
        title.push(Span::styled(
            format!("· {} ", truncate(&todo.title, title_width)),
            Style::default().fg(MUTED),
        ));
    }
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(if editing { ACCENT } else { MUTED }))
        .padding(Padding::horizontal(1))
        .title(Line::from(title))
        .title(Line::styled(hint, Style::default().fg(MUTED)).alignment(Alignment::Right));
    let content = block.inner(area);
    frame.render_widget(block, area);
    if editing {
        draw_multiline_editor(frame, &app.editor, Style::default().fg(TEXT), content);
        if app.editor.text().is_empty() {
            frame.render_widget(
                Paragraph::new("Write a note…").style(Style::default().fg(MUTED)),
                content,
            );
        }
        return;
    }
    let (text, color) = note_text(app);
    let width = usize::from(content.width);
    let mut lines = note_lines(text, width);
    if lines.len() > usize::from(content.height) {
        lines.truncate(usize::from(content.height));
        if let Some(last) = lines.last_mut() {
            *last = truncate(&format!("{last}…"), width).into_owned();
        }
    }
    frame.render_widget(
        Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<_>>())
            .style(Style::default().fg(color)),
        content,
    );
}

fn note_lines(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }
    text.split('\n')
        .flat_map(|line| {
            let wrapped = wrap(line, width);
            if wrapped.is_empty() {
                vec![String::new()]
            } else {
                wrapped
            }
        })
        .collect()
}

fn adding_task(app: &App) -> bool {
    app.vim.input() == Some(InputTarget::Task) && app.editing_id.is_none()
}

fn note_text(app: &App) -> (&str, ratatui::style::Color) {
    if adding_task(app) {
        ("Save the task before adding a note.", MUTED)
    } else if let Some(todo) = app.note_todo() {
        if todo.note.is_empty() {
            ("No note yet.", MUTED)
        } else {
            (todo.note.as_str(), TEXT)
        }
    } else {
        ("Select a task to see its note.", MUTED)
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/ui/notes.rs"]
mod tests;
