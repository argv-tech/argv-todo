use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

use super::{
    super::{
        text::truncate,
        theme::{ACCENT, MUTED},
    },
    render::draw_list,
};
use crate::{
    app::{App, TaskPane},
    config::TaskView,
};

pub(in crate::ui) fn draw_tasks(frame: &mut Frame, app: &mut App, area: Rect) {
    if app.task_view() == TaskView::Normal {
        draw_list(frame, app, area, TaskPane::Tree);
        return;
    }
    let horizontal = area.width >= 70;
    let layout = if horizontal {
        Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
    } else {
        Layout::vertical([
            Constraint::Percentage(50),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
    };
    let [todo, divider, completed] = layout.areas(area);
    let border = if horizontal {
        Borders::LEFT
    } else {
        Borders::TOP
    };
    let divider = if horizontal {
        Rect::new(
            divider.x + divider.width / 2,
            divider.y,
            u16::from(divider.width > 0),
            divider.height,
        )
    } else {
        divider
    };
    frame.render_widget(
        Block::default()
            .borders(border)
            .border_style(Style::default().fg(MUTED)),
        divider,
    );
    draw_pane(frame, app, todo, TaskPane::Todo, "Todo");
    draw_pane(frame, app, completed, TaskPane::Completed, "Completed");
}

fn draw_pane(frame: &mut Frame, app: &mut App, area: Rect, pane: TaskPane, title: &str) {
    if area.is_empty() {
        return;
    }
    let [heading, list] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
    let focused = app.task_pane() == pane;
    let title = format!(
        "{}{}",
        if focused { "› " } else { "  " },
        truncate(title, usize::from(area.width.saturating_sub(2)))
    );
    let style = if focused {
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(MUTED)
    };
    frame.render_widget(Paragraph::new(title).style(style), heading);
    draw_list(frame, app, list, pane);
}

#[cfg(test)]
#[path = "../../../tests/unit/ui/task_panes.rs"]
mod tests;
