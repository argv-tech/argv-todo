use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

use super::theme::{ACCENT, BG, MUTED, TEXT};

pub(super) fn draw_help(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let width = 54.min(area.width.saturating_sub(2));
    let height = 25.min(area.height.saturating_sub(2));
    let popup = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, popup);
    let lines = [
        "j/k       next / previous task",
        "l         select first child",
        "h         select parent",
        "gg / G    first / last task",
        "3j / 2k   repeat movement",
        "",
        "i/a       add child task",
        "o / O     add sibling below / above",
        "e / cc    edit task",
        "t         cycle priority",
        "ph/pm/pl  high / mid / low",
        "ctrl-p    cycle while typing",
        "Space/x   toggle task + children",
        "dd / 3dd  delete task tree(s)",
        "u         undo deletion",
        "",
        "/         search all tasks",
        "Esc       config / cancel / clear search",
        "Enter     save / apply search",
        "←/→       move input cursor",
        "Home/End  start / end of input",
        "ctrl-←/→  move by word",
        "ctrl-w/u  delete word / clear",
        "ctrl-r    reload tasks",
        "q/ctrl-c  quit",
        "",
        "j/k scroll · Esc close",
    ];
    let block = Block::default()
        .title(" keys ")
        .title_style(Style::default().fg(ACCENT))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MUTED))
        .style(Style::default().bg(BG).fg(TEXT));
    frame.render_widget(
        Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<_>>())
            .block(block)
            .scroll((app.help_scroll, 0))
            .wrap(Wrap { trim: true }),
        popup,
    );
}
