use ratatui::style::Color;

// ANSI colors follow the user's terminal palette; preserve its background.
pub(super) const BG: Color = Color::Reset;
pub(super) const TEXT: Color = Color::Reset;
pub(super) const MUTED: Color = Color::DarkGray;
pub(super) const ACCENT: Color = Color::Cyan;
pub(super) const ERROR: Color = Color::Red;
