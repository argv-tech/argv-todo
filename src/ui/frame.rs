use crate::{app::App, vim_motion::InputTarget};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Style,
    widgets::{Block, Paragraph, Wrap},
};

use super::{
    chrome::{draw_footer, draw_header, draw_input},
    config::draw_config,
    help::draw_help,
    layout::workspace,
    tasks::draw_tasks,
    theme::{ACCENT, BG, ERROR, TEXT},
};

pub(crate) fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    frame.render_widget(
        Block::default().style(Style::default().bg(BG).fg(TEXT)),
        area,
    );
    if area.width < 35 || area.height < 12 {
        frame.render_widget(
            Paragraph::new("argv-todo\n\nResize to 35 × 12 or larger.\nEsc back · Ctrl-c quit")
                .style(Style::default().fg(ACCENT))
                .wrap(Wrap { trim: true }),
            area,
        );
        return;
    }
    if app.configuring {
        draw_config(frame, app, workspace(area));
        return;
    }
    let show_input = app.vim.input() == Some(InputTarget::Search)
        || (app.vim.input().is_none() && !app.query.is_empty());
    let [header, _, body, message, input, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(u16::from(app.error)),
        Constraint::Length(u16::from(show_input)),
        Constraint::Length(1),
    ])
    .areas(workspace(area));
    draw_header(frame, app, header);
    draw_tasks(frame, app, body);
    if app.error {
        frame.render_widget(
            Paragraph::new(app.status.as_str()).style(Style::default().fg(ERROR)),
            message,
        );
    }
    draw_input(frame, app, input);
    draw_footer(frame, app, footer);
    if app.help {
        draw_help(frame, app);
    }
}
