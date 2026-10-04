use crate::db::Priority;
use ratatui::style::{Color, Modifier, Style};

// ANSI colors follow the user's terminal palette; preserve its background.
pub(super) const BG: Color = Color::Reset;
pub(super) const TEXT: Color = Color::Reset;
pub(super) const MUTED: Color = Color::DarkGray;
pub(super) const ACCENT: Color = Color::Cyan;
pub(super) const ERROR: Color = Color::Red;

pub(super) fn priority_style(priority: Priority) -> Style {
    Style::default().fg(match priority {
        Priority::High => Color::Red,
        Priority::Mid => Color::Yellow,
        Priority::Low => Color::Blue,
    })
}

pub(super) fn task_title_style(parent: bool, selected: bool, completed: bool) -> Style {
    let mut style = Style::default().fg(TEXT);
    if parent || selected {
        style = style.add_modifier(Modifier::BOLD);
    }
    if selected {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    if completed {
        style = style.add_modifier(Modifier::CROSSED_OUT);
    }
    style
}
