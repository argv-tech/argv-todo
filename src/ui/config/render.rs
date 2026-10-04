use super::super::theme::{ACCENT, ERROR, MUTED};
use super::{form::draw_form, paths::draw_paths};
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

pub(in crate::ui) const CONFIG_LOGO: [&str; 6] = [
    " █████╗ ██████╗  ██████╗ ██╗   ██╗      ████████╗ ██████╗ ██████╗  ██████╗ ",
    "██╔══██╗██╔══██╗██╔════╝ ██║   ██║      ╚══██╔══╝██╔═══██╗██╔══██╗██╔═══██╗",
    "███████║██████╔╝██║  ███╗██║   ██║█████╗   ██║   ██║   ██║██║  ██║██║   ██║",
    "██╔══██║██╔══██╗██║   ██║╚██╗ ██╔╝╚════╝   ██║   ██║   ██║██║  ██║██║   ██║",
    "██║  ██║██║  ██║╚██████╔╝ ╚████╔╝          ██║   ╚██████╔╝██████╔╝╚██████╔╝",
    "╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝   ╚═══╝           ╚═╝    ╚═════╝ ╚═════╝  ╚═════╝",
];

pub(in crate::ui) fn draw_config(frame: &mut Frame, app: &App, area: Rect) {
    let show_logo = area.height >= 17
        && CONFIG_LOGO
            .iter()
            .all(|line| unicode_width::UnicodeWidthStr::width(*line) <= usize::from(area.width));
    let spacious = area.height >= 17;
    let [logo, _, title, _, body, status, footer] = Layout::vertical([
        Constraint::Length(if show_logo { 6 } else { 1 }),
        Constraint::Length(u16::from(spacious)),
        Constraint::Length(1),
        Constraint::Length(u16::from(spacious || area.width >= 70)),
        Constraint::Min(3),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);
    let branding = if show_logo {
        CONFIG_LOGO.into_iter().map(Line::from).collect::<Vec<_>>()
    } else {
        vec![Line::from("argv-todo")]
    };
    frame.render_widget(
        Paragraph::new(branding)
            .alignment(Alignment::Center)
            .style(Style::default().fg(ACCENT)),
        logo,
    );
    frame.render_widget(
        Paragraph::new("configuration").style(Style::default().add_modifier(Modifier::BOLD)),
        title,
    );
    if area.width >= 70 {
        let [form, _, paths] = Layout::horizontal([
            Constraint::Percentage(42),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .areas(body);
        draw_form(frame, app, form, spacious);
        let divider = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(MUTED));
        let content = divider.inner(paths).inner(Margin {
            horizontal: 1,
            vertical: 0,
        });
        frame.render_widget(divider, paths);
        draw_paths(frame, app, content, false);
    } else {
        let [form, paths] = Layout::vertical([
            Constraint::Length(if spacious { 6 } else { 3 }),
            Constraint::Min(0),
        ])
        .areas(body);
        draw_form(frame, app, form, spacious);
        draw_paths(frame, app, paths, true);
    }
    frame.render_widget(
        Paragraph::new(app.status.as_str()).style(Style::default().fg(if app.error {
            ERROR
        } else {
            ACCENT
        })),
        status,
    );
    let hint = if app.vim.input().is_some() {
        super::super::chrome::draw_footer(frame, app, footer);
        return;
    } else if area.width < 45 {
        "Enter edit · Esc back · q quit"
    } else {
        "Enter/e edit · Esc tasks · q quit"
    };
    frame.render_widget(
        Paragraph::new(hint).style(Style::default().fg(MUTED)),
        footer,
    );
}
