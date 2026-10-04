use crate::{app::App, vim_motion::VimMode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::theme::{ACCENT, MUTED, TEXT};

pub(super) fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let [brand, summary] =
        Layout::horizontal([Constraint::Length(11), Constraint::Min(0)]).areas(area);
    frame.render_widget(
        Paragraph::new("argv-todo").style(Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        brand,
    );
    let done = app.todos.iter().filter(|todo| todo.done).count();
    let text = if app.todos.is_empty() {
        "0 tasks".into()
    } else {
        format!("{done}/{} done", app.todos.len())
    };
    frame.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Right)
            .style(Style::default().fg(MUTED)),
        summary,
    );
}

pub(super) fn draw_input(frame: &mut Frame, app: &App, area: Rect) {
    let input = area;
    if input.height == 0 {
        return;
    }
    if app.vim.mode() == VimMode::Normal {
        if !app.query.is_empty() {
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled("/ ", Style::default().fg(ACCENT)),
                    Span::raw(app.query.as_str()),
                    Span::styled("  · Esc clear", Style::default().fg(MUTED)),
                ])),
                input,
            );
        }
        return;
    }
    if app.vim.mode() != VimMode::Search {
        return;
    }
    let label = "/ ";
    let prefix_width = unicode_width::UnicodeWidthStr::width(label) as u16;
    let (text, column) = app
        .editor
        .viewport(input.width.saturating_sub(prefix_width));
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(label, Style::default().fg(ACCENT)),
            Span::raw(text),
        ])),
        input,
    );
    if !app.help && input.width > prefix_width {
        frame.set_cursor_position((input.x + prefix_width + column, input.y));
    }
}

pub(super) fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let editing = app.vim.mode() != VimMode::Normal;
    let pending = app.vim.pending_label();
    let [mode, keys] = Layout::horizontal([Constraint::Length(14), Constraint::Min(0)]).areas(area);
    frame.render_widget(
        Paragraph::new(format!(
            "{}  {pending}",
            app.vim.mode().label().to_lowercase()
        ))
        .style(Style::default().fg(if editing || !pending.is_empty() {
            ACCENT
        } else {
            MUTED
        })),
        mode,
    );
    let hint = if pending.ends_with('p') {
        if keys.width < 22 {
            "h high m mid l low"
        } else {
            "h high · m mid · l low"
        }
    } else if editing {
        if area.width < 45 {
            "Enter save  Esc"
        } else if area.width < 60 {
            "Enter save · Esc cancel"
        } else if app.vim.mode() == VimMode::Insert && area.width >= 80 {
            "Enter save · Esc cancel · Ctrl-p priority"
        } else {
            "Enter save  ·  Esc cancel  ·  ←/→ move"
        }
    } else if area.width < 45 {
        "i  Esc config  ?  q"
    } else if area.width < 75 {
        "i child · Esc config · ?"
    } else if area.width < 105 {
        "i child · o/O sibling · Esc config · ? help"
    } else {
        "j/k move · i/a child · o/O sibling · x done · t priority · Esc config · ? help · q quit"
    };
    frame.render_widget(
        Paragraph::new(hint)
            .alignment(Alignment::Right)
            .style(Style::default().fg(MUTED)),
        keys,
    );
}
