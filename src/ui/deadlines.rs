use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::theme::{ACCENT, BG, ERROR, MUTED, TEXT};
use crate::{
    app::{App, PickerFocus},
    db::month_len,
};

const SHORT_MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

pub(in crate::ui) fn draw_date_picker(frame: &mut Frame, app: &App) {
    let Some(picker) = &app.date_picker else {
        return;
    };
    let area = frame.area();
    // Live selection, weekday header, six week rows, and two hint rows inside borders.
    let (width, height) = (31, 12);
    if area.width < width || area.height < height {
        return;
    }
    let popup = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Pick deadline ")
        .border_style(Style::default().fg(ACCENT))
        .style(Style::default().bg(BG).fg(TEXT));
    let inner = block.inner(popup);
    frame.render_widget(Clear, popup);
    frame.render_widget(block, popup);
    let [echo, header, grid, hints] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(6),
        Constraint::Length(2),
    ])
    .areas(inner);
    let month = SHORT_MONTHS
        .get((picker.month.saturating_sub(1)) as usize)
        .unwrap_or(&"");
    let weekday = WEEKDAYS.get(picker.weekday_index() as usize).unwrap_or(&"");
    let time = Style::default().fg(TEXT);
    let focused = Style::default()
        .fg(ACCENT)
        .add_modifier(Modifier::BOLD | Modifier::REVERSED);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{weekday} {} {month} {}", picker.day, picker.year),
                Style::default().fg(TEXT),
            ),
            Span::styled(" · ", Style::default().fg(MUTED)),
            Span::styled(
                format!("{:02}", picker.hour),
                if picker.focus == PickerFocus::Hour {
                    focused
                } else {
                    time
                },
            ),
            Span::styled(":", Style::default().fg(MUTED)),
            Span::styled(
                format!("{:02}", picker.minute),
                if picker.focus == PickerFocus::Minute {
                    focused
                } else {
                    time
                },
            ),
        ]))
        .centered(),
        echo,
    );
    frame.render_widget(
        Paragraph::new("Su Mo Tu We Th Fr Sa")
            .style(Style::default().fg(MUTED))
            .centered(),
        header,
    );
    let today = crate::db::today();
    let selected = format!("{:04}-{:02}-{:02}", picker.year, picker.month, picker.day);
    // A full six-week grid: dimmed neighbours keep the popup stable across months.
    let dim = Style::default().fg(MUTED).add_modifier(Modifier::DIM);
    let (prev_year, prev_month) = if picker.month == 1 {
        (picker.year - 1, 12)
    } else {
        (picker.year, picker.month - 1)
    };
    let leading = picker.leading_blanks() as usize;
    let prev_len = month_len(prev_year, prev_month) as usize;
    let mut cells = Vec::with_capacity(42);
    for day in (prev_len - leading + 1)..=prev_len {
        cells.push(Span::styled(format!("{day:2} "), dim));
    }
    for day in 1..=month_len(picker.year, picker.month) {
        let iso = format!("{:04}-{:02}-{day:02}", picker.year, picker.month);
        let mut style = if iso.as_str() < today.as_str() {
            Style::default().fg(MUTED)
        } else {
            Style::default().fg(TEXT)
        };
        if iso == selected {
            style = style
                .fg(ACCENT)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED);
        }
        if iso == today {
            style = style.add_modifier(Modifier::UNDERLINED);
        }
        cells.push(Span::styled(format!("{day:2} "), style));
    }
    for day in 1..=(42 - cells.len()) {
        cells.push(Span::styled(format!("{day:2} "), dim));
    }
    let rows: Vec<Line> = cells
        .chunks(7)
        .map(|week| Line::from(week.to_vec()))
        .collect();
    frame.render_widget(Paragraph::new(rows), grid);
    if app.error {
        frame.render_widget(
            Paragraph::new(app.status.as_str()).style(Style::default().fg(ERROR)),
            hints,
        );
    } else {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from("h/l day · j/k week · H/L month"),
                Line::from("Tab time · t · x · Enter · Esc"),
            ])
            .style(Style::default().fg(MUTED)),
            hints,
        );
    }
}
