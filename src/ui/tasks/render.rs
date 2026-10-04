use crate::{
    app::{App, TaskPane},
    vim_motion::InputTarget,
};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
};

use super::super::{
    input::editor_spans,
    text::truncate,
    theme::{ACCENT, MUTED, TEXT, priority_style, task_title_style},
};
use super::rows::display_rows;

pub(super) fn draw_list(frame: &mut Frame, app: &mut App, area: Rect, pane: TaskPane) {
    if area.is_empty() {
        return;
    }
    let (visible, draft) = display_rows(app, pane);
    if visible.is_empty() {
        let query = if app.vim.input() == Some(InputTarget::Search) {
            app.editor.text()
        } else {
            &app.query
        };
        let (title, hint) = if !query.is_empty() {
            ("No matching tasks.", "Esc  clear search")
        } else if pane == TaskPane::Completed {
            ("No completed tasks.", "Tab  todo")
        } else if pane == TaskPane::Todo {
            ("No unfinished todos.", "i  add a task · Tab  completed")
        } else {
            ("Nothing on your list.", "i  add your first task")
        };
        let height = 3.min(area.height);
        let empty = Rect::new(
            area.x,
            area.y + area.height.saturating_sub(height) / 2,
            area.width,
            height,
        );
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(title, Style::default().fg(TEXT))),
                Line::from(""),
                Line::from(Span::styled(hint, Style::default().fg(ACCENT))),
            ])
            .alignment(Alignment::Center),
            empty,
        );
        return;
    }
    let child_counts: Vec<_> = visible
        .iter()
        .map(|row| {
            row.index
                .map_or((0, 0), |index| app.child_counts(app.todos[index].id))
        })
        .collect();
    let mut last_sibling = vec![true; visible.len()];
    let mut last_at_depth = Vec::new();
    for (row, task) in visible.iter().enumerate() {
        last_at_depth.truncate(task.depth + 1);
        if let Some(previous) = last_at_depth.get_mut(task.depth) {
            last_sibling[*previous] = false;
            *previous = row;
        } else {
            last_at_depth.push(row);
        }
    }
    let max_depth = (area.width.saturating_sub(18) as usize / 5).max(1);
    let mut ancestors = Vec::new();
    let mut state = *app.pane_list(pane);
    let focused = app.task_pane() == pane;
    if let Some(row) = draft {
        state.select(Some(row));
    } else if let Some(selected) = state.selected() {
        let row = visible
            .iter()
            .enumerate()
            .filter(|(_, row)| !row.ghost)
            .nth(selected)
            .map(|(row, _)| row);
        state.select(row);
    }
    let mut cursor = None;
    let items: Vec<_> = visible
        .iter()
        .enumerate()
        .map(|(row, tree_row)| {
            let todo = tree_row.index.map(|index| &app.todos[index]);
            let selected = focused && state.selected() == Some(row);
            let inline = focused
                && app.vim.input() == Some(InputTarget::Task)
                && todo.is_none_or(|todo| app.editing_id == Some(todo.id));
            let completed = todo.is_some_and(|todo| todo.done);
            let priority = if inline {
                app.input_priority
            } else {
                todo.map_or(app.input_priority, |todo| todo.priority)
            };
            let (done, total) = child_counts[row];
            let root = tree_row.depth == 0;
            let ghost_style = Style::default().fg(MUTED).add_modifier(Modifier::DIM);
            let style = if tree_row.ghost {
                ghost_style
            } else {
                task_title_style(root || total > 0, selected, completed && !inline)
            };
            let marker_style = if tree_row.ghost {
                ghost_style
            } else {
                Style::default().fg(if selected { ACCENT } else { TEXT })
            };
            let children = if tree_row.ghost {
                "  (parent)".into()
            } else if total == 0 {
                String::new()
            } else {
                format!("  {done}/{total}")
            };
            ancestors.truncate(tree_row.depth);
            let hidden = tree_row.depth.saturating_sub(max_depth);
            let mut guides = if hidden > 0 {
                "… ".into()
            } else {
                String::new()
            };
            for &continues in ancestors.iter().skip(hidden.max(1)) {
                guides.push_str(if continues { "│    " } else { "     " });
            }
            let branch = if root {
                String::new()
            } else {
                format!(
                    "{guides}{}",
                    if last_sibling[row] {
                        "└──  "
                    } else {
                        "├──  "
                    }
                )
            };
            ancestors.push(!last_sibling[row]);
            let prefix_width =
                2 + unicode_width::UnicodeWidthStr::width(branch.as_str()) as u16 + 8;
            let title = if inline {
                let (spans, column) =
                    editor_spans(&app.editor, area.width.saturating_sub(prefix_width), style);
                if prefix_width < area.width {
                    cursor = Some((row, prefix_width + column));
                }
                if app.editor.text().is_empty() {
                    vec![Span::styled("New task…", Style::default().fg(MUTED))]
                } else {
                    spans
                }
            } else {
                vec![Span::styled(
                    truncate(
                        todo.map_or("", |todo| todo.title.as_str()),
                        usize::from(area.width.saturating_sub(prefix_width)).saturating_sub(
                            unicode_width::UnicodeWidthStr::width(children.as_str()),
                        ),
                    ),
                    style,
                )]
            };
            let mut spans = vec![
                Span::styled(branch, marker_style),
                Span::styled(
                    if tree_row.ghost {
                        "·  "
                    } else if completed {
                        "✓  "
                    } else {
                        "□  "
                    },
                    marker_style,
                ),
                Span::styled(
                    format!("{:<4} ", priority.label()),
                    if tree_row.ghost {
                        ghost_style
                    } else {
                        priority_style(priority)
                    },
                ),
            ];
            spans.extend(title);
            spans.push(Span::styled(
                if inline { String::new() } else { children },
                if tree_row.ghost {
                    ghost_style
                } else {
                    Style::default().fg(MUTED)
                },
            ));
            ListItem::new(Line::from(spans))
        })
        .collect();
    let list = List::new(items).highlight_symbol(Span::styled(
        if focused { "› " } else { "· " },
        Style::default().fg(if focused { ACCENT } else { MUTED }),
    ));
    frame.render_stateful_widget(list, area, &mut state);
    *app.pane_list_mut(pane).offset_mut() = state.offset();
    if !app.help
        && let Some((row, column)) = cursor
        && row >= state.offset()
        && row - state.offset() < usize::from(area.height)
    {
        frame.set_cursor_position((area.x + column, area.y + (row - state.offset()) as u16));
    }
}
