use crate::{app::App, db::Priority, vim_motion::VimMode};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
};

use super::theme::{ACCENT, MUTED, TEXT};

struct DisplayRow {
    index: Option<usize>,
    depth: usize,
}

pub(super) fn draw_tasks(frame: &mut Frame, app: &mut App, area: Rect) {
    let mut visible: Vec<_> = app
        .visible_rows()
        .into_iter()
        .map(|row| DisplayRow {
            index: Some(row.index),
            depth: row.depth,
        })
        .collect();
    let adding = app.vim.mode() == VimMode::Insert && app.editing_id.is_none();
    let draft = if adding {
        let (start, end, depth) = app
            .adding_parent
            .and_then(|id| {
                visible.iter().enumerate().find_map(|(row, task)| {
                    (app.todos[task.index.unwrap()].id == id).then_some((row, task.depth))
                })
            })
            .map(|(parent, depth)| {
                let position = visible
                    .iter()
                    .enumerate()
                    .skip(parent + 1)
                    .find(|(_, task)| task.depth <= depth)
                    .map_or(visible.len(), |(row, _)| row);
                (parent + 1, position, depth + 1)
            })
            .unwrap_or((0, visible.len(), 0));
        let relative_position = app.adding_relative.and_then(|(id, above)| {
            app.todos
                .iter()
                .find(|todo| todo.id == id)
                .map(|todo| todo.position + i64::from(!above))
        });
        let position = (start..end)
            .find(|&row| {
                let todo = &app.todos[visible[row].index.unwrap()];
                visible[row].depth == depth
                    && (todo.priority > app.input_priority
                        || (todo.priority == app.input_priority
                            && relative_position.is_some_and(|position| todo.position >= position)))
            })
            .unwrap_or(end);
        visible.insert(position, DisplayRow { index: None, depth });
        Some(position)
    } else {
        None
    };
    if visible.is_empty() {
        let query = if app.vim.mode() == VimMode::Search {
            app.editor.text()
        } else {
            &app.query
        };
        let (title, hint) = if !query.is_empty() {
            ("No matching tasks.", "Esc  clear search")
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
    let mut state = app.list;
    if let Some(row) = draft {
        state.select(Some(row));
    }
    let mut cursor = None;
    let items: Vec<_> = visible
        .iter()
        .enumerate()
        .map(|(row, tree_row)| {
            let todo = tree_row.index.map(|index| &app.todos[index]);
            let selected = state.selected() == Some(row);
            let inline = app.vim.mode() == VimMode::Insert
                && todo.is_none_or(|todo| app.editing_id == Some(todo.id));
            let completed = todo.is_some_and(|todo| todo.done);
            let priority = if inline {
                app.input_priority
            } else {
                todo.unwrap().priority
            };
            let (done, total) = child_counts[row];
            let root = tree_row.depth == 0;
            let mut style = Style::default().fg(TEXT);
            if root || total > 0 || selected {
                style = style.add_modifier(Modifier::BOLD);
            }
            let children = if total == 0 {
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
            let title = if inline {
                let prefix_width =
                    2 + unicode_width::UnicodeWidthStr::width(branch.as_str()) as u16 + 8;
                let (text, column) = app.editor.viewport(area.width.saturating_sub(prefix_width));
                if prefix_width < area.width {
                    cursor = Some((row, prefix_width + column));
                }
                if app.editor.text().is_empty() {
                    Span::styled("New task…", Style::default().fg(MUTED))
                } else {
                    Span::styled(text, style)
                }
            } else {
                Span::styled(todo.unwrap().title.as_str(), style)
            };
            let line = Line::from(vec![
                Span::styled(
                    branch,
                    Style::default().fg(if selected { ACCENT } else { TEXT }),
                ),
                Span::styled(
                    if completed { "✓  " } else { "□  " },
                    Style::default().fg(if selected { ACCENT } else { TEXT }),
                ),
                Span::styled(
                    format!("{:<4} ", priority.label()),
                    Style::default().fg(match priority {
                        Priority::High => Color::Red,
                        Priority::Mid => Color::Yellow,
                        Priority::Low => Color::Blue,
                    }),
                ),
                title,
                Span::styled(
                    if inline { String::new() } else { children },
                    Style::default().fg(MUTED),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();
    let list = List::new(items).highlight_symbol(Span::styled("› ", Style::default().fg(ACCENT)));
    frame.render_stateful_widget(list, area, &mut state);
    *app.list.offset_mut() = state.offset();
    if !app.help
        && let Some((row, column)) = cursor
        && row >= state.offset()
        && row - state.offset() < usize::from(area.height)
    {
        frame.set_cursor_position((area.x + column, area.y + (row - state.offset()) as u16));
    }
}
