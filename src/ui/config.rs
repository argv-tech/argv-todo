use crate::{app::App, vim_motion::VimMode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::theme::{ACCENT, ERROR, MUTED};

pub(super) const CONFIG_LOGO: [&str; 6] = [
    " █████╗ ██████╗  ██████╗ ██╗   ██╗      ████████╗ ██████╗ ██████╗  ██████╗ ",
    "██╔══██╗██╔══██╗██╔════╝ ██║   ██║      ╚══██╔══╝██╔═══██╗██╔══██╗██╔═══██╗",
    "███████║██████╔╝██║  ███╗██║   ██║█████╗   ██║   ██║   ██║██║  ██║██║   ██║",
    "██╔══██║██╔══██╗██║   ██║╚██╗ ██╔╝╚════╝   ██║   ██║   ██║██║  ██║██║   ██║",
    "██║  ██║██║  ██║╚██████╔╝ ╚████╔╝          ██║   ╚██████╔╝██████╔╝╚██████╔╝",
    "╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝   ╚═══╝           ╚═╝    ╚═════╝ ╚═════╝  ╚═════╝",
];

pub(super) fn draw_config(frame: &mut Frame, app: &App, area: Rect) {
    let width = area.width.min(90);
    let area = Rect::new(
        area.x + (area.width - width) / 2,
        area.y,
        width,
        area.height,
    );
    let show_logo = area.height >= 17
        && CONFIG_LOGO
            .iter()
            .all(|line| unicode_width::UnicodeWidthStr::width(*line) <= usize::from(area.width));
    let spacious = area.height >= 17;
    let [logo, _, title, _, label, input, note, paths, status, footer] = Layout::vertical([
        Constraint::Length(if show_logo { 6 } else { 1 }),
        Constraint::Length(u16::from(spacious)),
        Constraint::Length(1),
        Constraint::Length(u16::from(spacious)),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(if spacious { 2 } else { 1 }),
        Constraint::Min(0),
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
    frame.render_widget(
        Paragraph::new("database_path").style(Style::default().fg(MUTED)),
        label,
    );
    let editing = app.vim.mode() != VimMode::Normal;
    let (value, column) = if editing {
        app.editor.viewport(input.width.saturating_sub(2))
    } else {
        (
            app.config
                .as_ref()
                .map_or("", |config| config.database_path.as_str()),
            0,
        )
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("› ", Style::default().fg(ACCENT)),
            Span::styled(value, Style::default().add_modifier(Modifier::BOLD)),
        ])),
        input,
    );
    if editing && input.width > 2 && input.height > 0 {
        frame.set_cursor_position((input.x + 2 + column, input.y));
    }
    frame.render_widget(
        Paragraph::new(if spacious {
            "Relative paths use the config folder.\nChanges apply on next launch."
        } else {
            "Applies on next launch."
        })
        .style(Style::default().fg(MUTED)),
        note,
    );
    if let Some(config) = &app.config {
        let mut lines = vec![
            Line::from(format!("config  {}", config.path.display())),
            Line::from(format!("using   {}", config.active_database.display())),
        ];
        if config.database_override {
            lines.push(Line::from("Current launch uses --db."));
        }
        frame.render_widget(
            Paragraph::new(lines).style(Style::default().fg(MUTED)),
            paths,
        );
    }
    frame.render_widget(
        Paragraph::new(app.status.as_str()).style(Style::default().fg(if app.error {
            ERROR
        } else {
            ACCENT
        })),
        status,
    );
    let hint = if editing {
        "Enter save · Esc cancel"
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
