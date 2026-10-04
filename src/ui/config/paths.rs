use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use super::super::{
    text::truncate,
    theme::{ACCENT, MUTED},
};

pub(super) fn draw_paths(frame: &mut Frame, app: &App, area: Rect, compact: bool) {
    let Some(config) = &app.config else {
        return;
    };
    let config_path = config.path.display().to_string();
    let database_path = config.active_database.display().to_string();
    let lines = if compact {
        let width = usize::from(area.width.saturating_sub(8));
        let mut lines = vec![
            Line::from(vec![
                Span::styled("config  ", Style::default().fg(MUTED)),
                Span::raw(truncate(&config_path, width)),
            ]),
            Line::from(vec![
                Span::styled("using   ", Style::default().fg(MUTED)),
                Span::raw(truncate(&database_path, width)),
            ]),
        ];
        if config.database_override {
            lines.push(Line::from(Span::styled(
                "Current launch uses --db.",
                Style::default().fg(ACCENT),
            )));
        }
        lines
    } else {
        let heading = Style::default().add_modifier(Modifier::BOLD);
        let dense = area.height < 9;
        let width = if dense {
            usize::from(area.width)
        } else {
            usize::MAX
        };
        let mut lines = vec![
            Line::from(Span::styled("active database", heading)),
            Line::from(truncate(&database_path, width)),
        ];
        if !dense {
            lines.push(Line::default());
        }
        lines.extend([
            Line::from(Span::styled("configuration file", heading)),
            Line::from(truncate(&config_path, width)),
        ]);
        if config.database_override {
            if !dense {
                lines.push(Line::default());
            }
            lines.push(Line::from(Span::styled(
                "Current launch uses --db.",
                Style::default().fg(ACCENT),
            )));
        }
        lines
    };
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}
