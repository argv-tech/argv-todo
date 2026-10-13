use crate::{app::App, config::TaskView, vim_motion::InputTarget};
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
    projects::{draw_picker, draw_tabs},
    tasks::{draw_note, draw_tasks, note_height},
    theme::{ACCENT, BG, ERROR, TEXT},
};

pub(crate) fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    frame.render_widget(
        Block::default().style(Style::default().bg(BG).fg(TEXT)),
        area,
    );
    if app.help {
        draw_help(frame, app);
        return;
    }
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
    let show_input = matches!(
        app.vim.input(),
        Some(InputTarget::Search | InputTarget::ProjectName)
    ) || (app.vim.input().is_none() && !app.query.is_empty());
    let workspace = workspace(area);
    let reserved_rows = 3 + u16::from(app.error) + u16::from(show_input);
    let note_area = ratatui::layout::Rect {
        height: workspace.height.saturating_sub(reserved_rows),
        ..workspace
    };
    let [header, tabs, body, note, message, input, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(note_height(app, note_area)),
        Constraint::Length(u16::from(app.error)),
        Constraint::Length(u16::from(show_input)),
        Constraint::Length(1),
    ])
    .areas(workspace);
    draw_header(frame, app, header);
    if app.task_view() == TaskView::Projects {
        draw_tabs(frame, app, tabs);
    }
    draw_tasks(frame, app, body);
    draw_note(frame, app, note);
    if app.error {
        frame.render_widget(
            Paragraph::new(app.status.as_str()).style(Style::default().fg(ERROR)),
            message,
        );
    }
    draw_input(frame, app, input);
    draw_footer(frame, app, footer);
    draw_picker(frame, app);
}
