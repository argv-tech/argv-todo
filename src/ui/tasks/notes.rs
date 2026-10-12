use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::super::{
    input::draw_editor,
    theme::{ACCENT, MUTED, TEXT},
};
use crate::{app::App, vim_motion::InputTarget};

pub(in crate::ui) fn draw_note(frame: &mut Frame, app: &App, area: Rect) {
    if area.is_empty() {
        return;
    }
    let editing = app.vim.input() == Some(InputTarget::Note);
    let heading = if editing {
        " › Note · Enter save "
    } else {
        " Note · n edit "
    };
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(MUTED))
        .title(heading)
        .title_style(
            Style::default()
                .fg(if editing { ACCENT } else { MUTED })
                .add_modifier(Modifier::BOLD),
        );
    let content = block.inner(area);
    frame.render_widget(block, area);
    if editing {
        draw_editor(
            frame,
            &app.editor,
            "",
            Style::default().fg(TEXT),
            content,
            true,
        );
        return;
    }
    let adding = app.vim.input() == Some(InputTarget::Task) && app.editing_id.is_none();
    let (text, color) = if adding {
        ("Save the task before adding a note.", MUTED)
    } else if let Some(todo) = app.selected_todo() {
        if todo.note.is_empty() {
            ("No note yet.", MUTED)
        } else {
            (todo.note.as_str(), TEXT)
        }
    } else {
        ("Select a task to see its note.", MUTED)
    };
    frame.render_widget(
        Paragraph::new(text)
            .style(Style::default().fg(color))
            .wrap(Wrap { trim: false }),
        content,
    );
}

#[cfg(test)]
#[path = "../../../tests/unit/ui/notes.rs"]
mod tests;
