use super::super::{
    input::draw_editor,
    text::truncate,
    theme::{ACCENT, MUTED},
};
use crate::app::{App, ConfigSetting};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

pub(super) fn draw_form(frame: &mut Frame, app: &App, area: Rect) {
    if area.is_empty() {
        return;
    }
    let total = ConfigSetting::ALL.len() + 1;
    let height = usize::from(area.height).min(total);
    let selected = if app.config_setting == ConfigSetting::DatabasePath {
        1
    } else {
        app.config_setting.index() + 1
    };
    let offset = (selected + 1)
        .saturating_sub(height)
        .min(total.saturating_sub(height));
    let style = Style::default().add_modifier(Modifier::BOLD);
    for (visible_row, row) in (offset..total).take(height).enumerate() {
        let line_area = Rect::new(area.x, area.y + visible_row as u16, area.width, 1);
        if row == 0 {
            frame.render_widget(
                Paragraph::new("database_path").style(Style::default().fg(MUTED)),
                line_area,
            );
            continue;
        }
        let setting = if row == 1 {
            ConfigSetting::DatabasePath
        } else {
            ConfigSetting::ALL[row - 1]
        };
        if setting == ConfigSetting::DatabasePath && app.vim.input().is_some() {
            draw_editor(frame, &app.editor, "› ", style, line_area, true);
            continue;
        }
        let focused = setting == app.config_setting;
        let value_style = if focused {
            style.add_modifier(Modifier::UNDERLINED)
        } else {
            style
        };
        let mut spans = vec![Span::styled(
            if focused { "› " } else { "  " },
            Style::default().fg(ACCENT),
        )];
        let label_width = if setting == ConfigSetting::DatabasePath {
            0
        } else {
            spans.push(Span::styled(
                format!("{}  ", setting.name()),
                Style::default().fg(MUTED),
            ));
            setting.name().len() + 2
        };
        spans.push(Span::styled(
            truncate(
                setting.value(app),
                usize::from(area.width).saturating_sub(label_width + 2),
            ),
            value_style,
        ));
        frame.render_widget(Paragraph::new(Line::from(spans)), line_area);
    }
    if area.height >= total as u16 + 3 {
        let note = Rect::new(area.x, area.y + total as u16 + 1, area.width, 2);
        frame.render_widget(
            Paragraph::new("Database path: next launch.\nOther settings: apply immediately.")
                .style(Style::default().fg(MUTED)),
            note,
        );
    }
}
