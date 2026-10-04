use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    text::Line,
    widgets::Paragraph,
};

use super::theme::ACCENT;

pub(super) const LOGO: [&str; 6] = [
    " █████╗ ██████╗  ██████╗ ██╗   ██╗      ████████╗ ██████╗ ██████╗  ██████╗ ",
    "██╔══██╗██╔══██╗██╔════╝ ██║   ██║      ╚══██╔══╝██╔═══██╗██╔══██╗██╔═══██╗",
    "███████║██████╔╝██║  ███╗██║   ██║█████╗   ██║   ██║   ██║██║  ██║██║   ██║",
    "██╔══██║██╔══██╗██║   ██║╚██╗ ██╔╝╚════╝   ██║   ██║   ██║██║  ██║██║   ██║",
    "██║  ██║██║  ██║╚██████╔╝ ╚████╔╝          ██║   ╚██████╔╝██████╔╝╚██████╔╝",
    "╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝   ╚═══╝           ╚═╝    ╚═════╝ ╚═════╝  ╚═════╝",
];

pub(super) fn height(area: Rect) -> u16 {
    if area.height >= 17
        && LOGO
            .iter()
            .all(|line| unicode_width::UnicodeWidthStr::width(*line) <= usize::from(area.width))
    {
        6
    } else {
        1
    }
}

pub(super) fn draw(frame: &mut Frame, area: Rect) {
    let lines = if area.height >= 6 {
        LOGO.into_iter().map(Line::from).collect::<Vec<_>>()
    } else {
        vec![Line::from("argv-todo")]
    };
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .style(Style::default().fg(ACCENT)),
        area,
    );
}
