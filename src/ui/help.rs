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

pub(crate) const HELP_LINES: &[&str] = &[
    "TASK TREE",
    "j/k       next / previous",
    "l/h       first child / parent",
    "gg / G    first / last task",
    "3j / 2k   repeat movement",
    "i/a       add child task",
    "o / O     sibling below / above",
    "e / cc    edit task",
    "t         cycle priority",
    "ph/pm/pl  high / mid / low",
    "Space/x/Enter  toggle subtree",
    "dd / 3dd  delete task tree(s)",
    "u         undo deletion",
    "/         search all tasks",
    "Esc       config / clear search",
    "ctrl-r    reload tasks",
    "",
    "ALL INPUT FIELDS",
    "Enter     save/apply any mode",
    "Esc       return to Normal",
    "Esc again cancel field",
    "i/a/I/A   insert at cursor/edges",
    "h/l       left/right or arrows",
    "0 / ^ / $ start / text / end",
    "w/b/e     word start/back/end",
    "W/B/E     space-separated WORDs",
    "ge/gE     previous word end",
    "gg/G      field start/end",
    "3| / %    column / bracket",
    "f/F/t/T + char  find/till",
    "; / ,     repeat/reverse find",
    "d/c/y     delete/change/yank",
    "dd/cc/yy  whole field",
    "2d3w      multiply counts",
    "iw/aw     inner/around word",
    "iW/aW     inner/around WORD",
    "i/a + () [] {} <> or quotes",
    "Quotes: single/double/backtick",
    "di{ / da} inside/with braces",
    "v / V     Visual / whole field",
    "viw       select word",
    "o         swap selection ends",
    "d/c/y     act on selection",
    "x/X/D/C/s/S  delete/change",
    "r2 / 3r2  replace with 2",
    "R         Replace mode",
    "p/P       put after/before",
    "u/ctrl-r  undo/redo field edit",
    ".         repeat field change",
    "gu/gU/g~ + motion  change case",
    "u/U/~     Visual case change",
    "ctrl-←/→  move by word",
    "ctrl-w/u  delete word / clear",
    "ctrl-p    task input priority",
    "Paste     insert/replace text",
    "",
    "q tree / ctrl-c anywhere quit",
];

pub(super) fn draw_help(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let width = 54.min(area.width.saturating_sub(2));
    let height = 25.min(area.height.saturating_sub(2));
    let popup = centered(area, width, height);
    frame.render_widget(Clear, popup);
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
        Paragraph::new(
            HELP_LINES
                .iter()
                .copied()
                .map(Line::from)
                .collect::<Vec<_>>(),
        )
        .scroll((app.help_scroll, 0))
        .wrap(Wrap { trim: true }),
        content,
    );
    frame.render_widget(
        Paragraph::new("j/k scroll · Esc close").style(Style::default().fg(ACCENT)),
        footer,
    );
}
