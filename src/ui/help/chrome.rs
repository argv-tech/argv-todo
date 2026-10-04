use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::{
    super::{
        text::truncate,
        theme::{ACCENT, MUTED, TEXT},
    },
    content::Document,
};
use crate::app::App;

pub(super) fn draw_sections(frame: &mut Frame, document: &Document, app: &App, area: Rect) {
    let [heading, _, entries] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(area);
    frame.render_widget(
        Paragraph::new("SHORTCUT GUIDE")
            .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        heading,
    );
    let active = app.help_view.selected_section();
    let mut lines = Vec::new();
    for (index, section) in document.sections.iter().enumerate() {
        if index > 0 {
            lines.push(Line::default());
        }
        let selected = index == active;
        let style = if selected {
            Style::default()
                .fg(ACCENT)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD)
        };
        lines.push(Line::from(vec![
            Span::styled(
                if selected { "› " } else { "  " },
                Style::default().fg(ACCENT),
            ),
            Span::styled(
                truncate(
                    &section.title.to_uppercase(),
                    usize::from(area.width.saturating_sub(2)),
                )
                .into_owned(),
                style,
            ),
        ]));
    }
    let height = usize::from(entries.height);
    let selected_row = active * 2;
    let offset = (selected_row + 1)
        .saturating_sub(height)
        .min(lines.len().saturating_sub(height));
    frame.render_widget(Paragraph::new(lines).scroll((offset as u16, 0)), entries);
}

pub(super) fn draw_footer(frame: &mut Frame, app: &App, height: u16, area: Rect) {
    let view = &app.help_view;
    let [scroll, close] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);
    let start = if view.total_rows() == 0 {
        0
    } else {
        usize::from(view.offset()) + 1
    };
    let end = (usize::from(view.offset()) + usize::from(height)).min(view.total_rows());
    let position = format!("{start}-{end}/{}", view.total_rows());
    let position_width = if scroll.width >= 26 {
        position.len() as u16 + 2
    } else {
        0
    };
    let [hint, progress] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(position_width)]).areas(scroll);
    let candidates: &[&str] = if view.has_guide() {
        &[
            "j/k section · Tab next · PgUp/PgDn scroll",
            "j/k section · Tab next",
            "j/k section",
            "j/k",
        ]
    } else {
        &[
            "j/k scroll · PgUp/PgDn page · gg/G ends",
            "j/k scroll · PgUp/PgDn page",
            "j/k scroll",
            "j/k",
        ]
    };
    let hint_text = candidates
        .iter()
        .copied()
        .find(|text| Line::from(*text).width() <= usize::from(hint.width))
        .unwrap_or("");
    frame.render_widget(
        Paragraph::new(hint_text).style(Style::default().fg(MUTED)),
        hint,
    );
    frame.render_widget(
        Paragraph::new(Line::from(position).right_aligned()).style(Style::default().fg(MUTED)),
        progress,
    );
    let close_hint = if close.width >= 13 {
        "Esc / ? close"
    } else if close.width >= 9 {
        "Esc close"
    } else {
        "Esc"
    };
    frame.render_widget(
        Paragraph::new(close_hint).style(Style::default().fg(ACCENT)),
        close,
    );
}
