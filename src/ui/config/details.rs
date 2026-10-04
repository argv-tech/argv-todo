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
        setting => draw_preference(frame, app, setting, area),
    }
}

fn draw_preference(frame: &mut Frame, app: &App, setting: ConfigSetting, area: Rect) {
    let description = match setting {
        ConfigSetting::DefaultPriority => "Priority for new root tasks.",
        ConfigSetting::ShowCompleted if app.task_view() == TaskView::Split => {
            "Always false in Split."
        }
        ConfigSetting::ShowCompleted => "Show done tasks in Normal view.",
        ConfigSetting::SortOrder => "Order siblings; keep subtrees.",
        ConfigSetting::ShowHints => "Show footer shortcuts.",
        _ => return,
    };
    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{} · ", setting.name()),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                setting.value(app),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(description),
    ];
    if area.height >= 4 {
        let details: &[&str] = match setting {
            ConfigSetting::DefaultPriority => &[
                "Children inherit their parent.",
                "Existing priorities stay intact.",
                "Enter / h / l cycles high/mid/low.",
            ],
            ConfigSetting::ShowCompleted if app.task_view() == TaskView::Split => &[
                "Done tasks use the Completed pane.",
                "Your Normal preference is kept.",
            ],
            ConfigSetting::ShowCompleted => &[
                "Hidden tasks remain in SQLite.",
                "Enter / h / l toggles visibility.",
            ],
            ConfigSetting::SortOrder => &[
                "priority: highest priority first.",
                "manual: use Shift+H/J/K/L.",
                "J/K moves down/up among siblings.",
                "H outdents; L indents.",
                "Enter / h / l changes ordering.",
            ],
            ConfigSetting::ShowHints => &[
                "Mode and errors stay visible.",
                "Help is available with ?.",
                "Enter / h / l toggles hints.",
            ],
            _ => &[],
        };
        lines.push(Line::default());
        lines.extend(details.iter().map(|&text| Line::from(text)));
    }
    frame.render_widget(Paragraph::new(lines), area);
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
