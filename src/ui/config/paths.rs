use crate::{app::App, vim_motion::InputTarget};
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
    let value = if app.vim.input() == Some(InputTarget::DatabasePath) {
        app.editor.text()
    } else {
        config.database_path.as_str()
    };
    let resolved = config.resolve_database_path(value);
    let preview = (!value.trim().is_empty() && resolved != config.active_database)
        .then(|| resolved.display().to_string());
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
        if let Some(preview) = &preview {
            lines.push(Line::from(vec![
                Span::styled("preview (after restart) ", Style::default().fg(MUTED)),
                Span::raw(truncate(
                    preview,
                    usize::from(area.width.saturating_sub(24)),
                )),
            ]));
        }
        if config.database_override {
            lines.push(Line::from(Span::styled(
                "Current launch uses --db.",
                Style::default().fg(ACCENT),
            )));
        }
        lines
    } else {
        let heading = Style::default().add_modifier(Modifier::BOLD);
        let dense = area.height < if preview.is_some() { 12 } else { 9 };
        let inline_details =
            preview.is_some() && area.height < 6 + u16::from(config.database_override);
        let width = if dense {
            usize::from(area.width)
        } else {
            usize::MAX
        };
        let mut lines = vec![
            Line::from(Span::styled("active database", heading)),
            Line::from(truncate(&database_path, width)),
        ];
        if let Some(preview) = &preview {
            if !dense {
                lines.push(Line::default());
            }
            if inline_details {
                lines.push(Line::from(vec![
                    Span::styled("preview (after restart) ", heading),
                    Span::raw(truncate(
                        preview,
                        usize::from(area.width.saturating_sub(24)),
                    )),
                ]));
            } else {
                lines.extend([
                    Line::from(Span::styled("database preview (after restart)", heading)),
                    Line::from(truncate(preview, width)),
                ]);
            }
        }
        if !dense {
            lines.push(Line::default());
        }
        if inline_details {
            lines.push(Line::from(vec![
                Span::styled("configuration file  ", heading),
                Span::raw(truncate(
                    &config_path,
                    usize::from(area.width.saturating_sub(20)),
                )),
            ]));
        } else {
            lines.extend([
                Line::from(Span::styled("configuration file", heading)),
                Line::from(truncate(&config_path, width)),
            ]);
        }
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
