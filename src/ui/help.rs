use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Style,
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

use super::{
    layout::centered,
    theme::{ACCENT, BG, MUTED, TEXT},
};

pub(super) fn draw_help(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let width = 54.min(area.width.saturating_sub(2));
    let height = 25.min(area.height.saturating_sub(2));
    let popup = centered(area, width, height);
    frame.render_widget(Clear, popup);
    let lines = [
        "j/k       next / previous",
        "l         select first child",
        "h         select parent",
        "gg / G    first / last task",
        "3j / 2k   repeat movement",
        "",
        "i/a       add child task",
        "o / O     sibling below / above",
        "e / cc    edit task",
        "t         cycle priority",
        "ph/pm/pl  high / mid / low",
        "ctrl-p    priority while typing",
        "Space/x/Enter  toggle subtree",
        "dd / 3dd  delete task tree(s)",
        "u         undo deletion",
        "",
        "/         search all tasks",
        "Esc       config / cancel / clear",
        "Enter     save / apply search",
        "←/→       move input cursor",
        "Home/End  input start / end",
        "ctrl-←/→  move by word",
        "ctrl-w/u  delete word / clear",
        "ctrl-r    reload tasks",
        "q/ctrl-c  quit",
    ];
    let block = Block::default()
        .title(" keys ")
        .title_style(Style::default().fg(ACCENT))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MUTED))
        .style(Style::default().bg(BG).fg(TEXT));
    let [content, footer] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(block.inner(popup));
    frame.render_widget(block, popup);
    frame.render_widget(
        Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<_>>())
            .scroll((app.help_scroll, 0))
            .wrap(Wrap { trim: true }),
        content,
    );
    frame.render_widget(
        Paragraph::new("j/k scroll · Esc close").style(Style::default().fg(ACCENT)),
        footer,
    );
}
