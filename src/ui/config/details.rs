use crate::{
    app::{App, ConfigSetting},
    config::TaskView,
};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::{
    super::theme::{ACCENT, MUTED},
    paths::draw_paths,
};

pub(super) fn draw_details(frame: &mut Frame, app: &App, area: Rect, compact: bool) {
    match app.config_setting {
        ConfigSetting::DatabasePath => draw_paths(frame, app, area, compact),
        ConfigSetting::TaskView => draw_task_view(frame, app.task_view(), area),
    }
}

fn draw_task_view(frame: &mut Frame, view: TaskView, area: Rect) {
    let heading = Style::default().add_modifier(Modifier::BOLD);
    let muted = Style::default().fg(MUTED);
    let description = match view {
        TaskView::Normal => "Full tree, including done tasks.",
        TaskView::Split => "Todo / Completed · Tab switches",
    };
    let mut lines = vec![
        Line::from(vec![
            Span::styled("task view · ", heading),
            Span::styled(view.label(), heading.fg(ACCENT)),
            Span::styled(" (live)", muted),
        ]),
        Line::from(description),
        Line::from(Span::styled("Enter / h / l changes layout.", muted)),
    ];
    if area.height >= 9 && area.width >= 33 {
        lines.extend([
            Line::default(),
            Line::from(Span::styled("layout preview", heading)),
        ]);
        match view {
            TaskView::Normal => lines.extend([
                Line::from(Span::styled("Tasks", heading)),
                Line::from("○ Parent"),
                Line::from("  ├─ ○ Todo"),
                Line::from("  └─ ✓ Completed"),
            ]),
            TaskView::Split => {
                lines.extend([
                    Line::from(Span::styled("Todo             Completed", heading)),
                    Line::from(vec![
                        Span::raw("○ Parent         "),
                        Span::styled("Parent (parent)", muted.add_modifier(Modifier::DIM)),
                    ]),
                    Line::from("  └─ ○ Todo        └─ ✓ Completed"),
                ]);
                if area.height >= 11 {
                    lines.extend([
                        Line::default(),
                        Line::from(Span::styled("Ghost parents provide context.", muted)),
                        Line::from(Span::styled("Panes stack in narrow terminals.", muted)),
                    ]);
                }
            }
        }
    }
    frame.render_widget(Paragraph::new(lines), area);
}
