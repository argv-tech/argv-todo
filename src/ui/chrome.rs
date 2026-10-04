use crate::{
    app::App,
    config::TaskView,
    vim_motion::{InputTarget, VimMode},
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::{
    input::draw_editor,
    text::truncate,
    theme::{ACCENT, MUTED, TEXT},
};
use unicode_width::UnicodeWidthStr;

pub(super) fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let brand = Line::from(vec![
        Span::styled(
            "argv-todo",
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            concat!(" v", env!("CARGO_PKG_VERSION")),
            Style::default().fg(Color::Red),
        ),
    ]);
    let brand_width = (brand.width() + 2) as u16;
    let [brand_area, summary] =
        Layout::horizontal([Constraint::Length(brand_width), Constraint::Min(0)]).areas(area);
    frame.render_widget(Paragraph::new(brand), brand_area);
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
    if area.is_empty() {
        return;
    }
    if app.vim.input().is_none() {
        if !app.query.is_empty() {
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled("/ ", Style::default().fg(ACCENT)),
                    Span::raw(truncate(
                        &app.query,
                        usize::from(area.width.saturating_sub(14)),
                    )),
                    Span::styled("  · Esc clear", Style::default().fg(MUTED)),
                ])),
                area,
            );
        }
        return;
    }
    if app.vim.input() != Some(InputTarget::Search) {
        return;
    }
    draw_editor(frame, &app.editor, "/ ", Style::default(), area, !app.help);
}

pub(super) fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let editing = app.vim.input().is_some();
    let pending = app.vim.pending_label();
    let label = format!("{}  {pending}", app.vim.mode().label().to_lowercase());
    let label = label.trim_end();
    let mode_width = (label.width() + 2).min(usize::from(area.width) / 2) as u16;
    let [mode, keys] =
        Layout::horizontal([Constraint::Length(mode_width), Constraint::Min(0)]).areas(area);
    frame.render_widget(
        Paragraph::new(label).style(Style::default().fg(if editing || !pending.is_empty() {
            ACCENT
        } else {
            MUTED
        })),
        mode,
    );
    let hint = footer_hint(app, usize::from(keys.width));
    frame.render_widget(
        Paragraph::new(hint)
            .alignment(Alignment::Right)
            .style(Style::default().fg(MUTED)),
        keys,
    );
}

fn footer_hint(app: &App, width: usize) -> &'static str {
    let candidates: &[&str] = if app.vim.input().is_some() && app.vim.mode().visual() {
        &[
            "d delete · c change · y yank · Esc normal",
            "d/c/y · Esc normal",
            "d/c/y · Esc",
        ]
    } else if app.vim.input().is_some() && app.vim.mode() == VimMode::Normal {
        if app.vim.input() == Some(InputTarget::Search) {
            &[
                "i/a insert · v select · Enter apply · Esc cancel",
                "i/a · v · Enter apply · Esc",
                "i · v · Enter · Esc",
            ]
        } else {
            &[
                "i/a insert · v select · Enter save · Esc cancel",
                "i/a · v · Enter save · Esc cancel",
                "i · v · Enter · Esc",
            ]
        }
    } else if app.vim.input().is_none() && app.vim.pending_label().ends_with('p') {
        &[
            "h high · m mid · l low",
            "h high m mid l low",
            "h/m/l priority",
            "h/m/l",
        ]
    } else if app.vim.input() == Some(InputTarget::Search) {
        &[
            "Enter apply · Esc normal · ←/→ move",
            "Enter apply · Esc normal",
            "Enter apply · Esc",
            "Enter · Esc",
        ]
    } else if app.vim.input() == Some(InputTarget::Task) {
        &[
            "Enter save · Esc normal · Ctrl-p priority",
            "Enter save · Esc normal",
            "Enter save · Esc",
            "Enter · Esc",
        ]
    } else if app.vim.input().is_some() {
        &[
            "Enter save · Esc normal · ←/→ move",
            "Enter save · Esc normal",
            "Enter save · Esc",
            "Enter · Esc",
        ]
    } else if !app.query.is_empty() {
        &[
            "j/k move · i child · Esc clear · ? help · q quit",
            "Esc clear · ? help · q quit",
            "Esc clear · ? · q",
        ]
    } else if app.task_view() != TaskView::Normal {
        &[
            "Tab pane · j/k move · h/l tree · i child · Esc config · ? help · q quit",
            "Tab pane · i child · Esc config · ? help · q quit",
            "Tab pane · Esc config · ? · q",
            "Tab pane · ? · q",
            "Tab · ? · q",
        ]
    } else {
        &[
            "j/k move · h/l tree · i/a child · o/O sibling · x done · t priority · Esc config · ? help · q quit",
            "i child · o/O sibling · Esc config · ? help · q quit",
            "i child · Esc config · ? help · q quit",
            "i · Esc config · ? · q",
            "i · ? · q",
        ]
    };
    candidates
        .iter()
        .copied()
        .find(|hint| hint.width() <= width)
        .unwrap_or("")
}
