use super::super::{
    input::draw_editor,
    text::truncate,
    theme::{ACCENT, MUTED},
};
use crate::app::{App, ConfigCategory, ConfigSetting};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

enum FormRow {
    Category(ConfigCategory),
    DatabaseLabel,
    Setting(ConfigSetting),
    Gap,
}

fn rows() -> Vec<FormRow> {
    let mut rows = Vec::new();
    let mut category = None;
    for setting in ConfigSetting::ALL {
        if category != Some(setting.category()) {
            if category.is_some() {
                rows.push(FormRow::Gap);
            }
            category = Some(setting.category());
            rows.push(FormRow::Category(setting.category()));
        }
        if setting == ConfigSetting::DatabasePath {
            rows.push(FormRow::DatabaseLabel);
        }
        rows.push(FormRow::Setting(setting));
    }
    rows
}

pub(super) fn form_height() -> u16 {
    rows().len() as u16
}

fn draw_category(frame: &mut Frame, category: ConfigCategory, area: Rect) {
    let title = match category {
        ConfigCategory::Storage => "STORAGE",
        ConfigCategory::Display => "DISPLAY",
        ConfigCategory::TaskBehavior => "TASK BEHAVIOR",
    };
    frame.render_widget(
        Paragraph::new(title).style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        area,
    );
}

pub(super) fn draw_form(frame: &mut Frame, app: &App, area: Rect) {
    if area.is_empty() {
        return;
    }
    let rows = rows();
    let total = rows.len();
    let height = usize::from(area.height).min(total);
    let selected = rows
        .iter()
        .position(|row| matches!(row, FormRow::Setting(setting) if *setting == app.config_setting))
        .unwrap_or(0);
    let category_row = rows[..selected]
        .iter()
        .rposition(|row| matches!(row, FormRow::Category(_)))
        .unwrap_or(0);
    let mut offset = (selected + 1)
        .saturating_sub(height)
        .min(total.saturating_sub(height));
    // Keep the selected setting's category visible when its heading scrolls away.
    let pinned_category = offset > category_row && height > 1;
    let content_height = height - usize::from(pinned_category);
    if pinned_category {
        draw_category(
            frame,
            app.config_setting.category(),
            Rect::new(area.x, area.y, area.width, 1),
        );
        offset = (selected + 1)
            .saturating_sub(content_height)
            .max(category_row + 1);
    }
    let style = Style::default().add_modifier(Modifier::BOLD);
    for (visible_row, row) in rows.iter().skip(offset).take(content_height).enumerate() {
        let line_area = Rect::new(
            area.x,
            area.y + visible_row as u16 + u16::from(pinned_category),
            area.width,
            1,
        );
        let setting = match row {
            FormRow::Category(category) => {
                draw_category(frame, *category, line_area);
                continue;
            }
            FormRow::DatabaseLabel => {
                frame.render_widget(
                    Paragraph::new("database_path").style(Style::default().fg(MUTED)),
                    line_area,
                );
                continue;
            }
            FormRow::Gap => continue,
            FormRow::Setting(setting) => *setting,
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
