use crate::{app::App, db::Priority, vim_motion::vim_mode::VimMode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

// ANSI colors follow the user's terminal palette; preserve its background.
const BG: Color = Color::Reset;
const TEXT: Color = Color::Reset;
const MUTED: Color = Color::DarkGray;
const ACCENT: Color = Color::Cyan;
const DONE: Color = Color::Green;
const ERROR: Color = Color::Red;

fn workspace(area: Rect) -> Rect {
    let margin = if area.width >= 60 { 2 } else { 1 };
    Rect::new(
        area.x + margin,
        area.y + 1,
        area.width.saturating_sub(margin * 2),
        area.height.saturating_sub(2),
    )
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    frame.render_widget(
        Block::default().style(Style::default().bg(BG).fg(TEXT)),
        area,
    );
    if area.width < 35 || area.height < 12 {
        frame.render_widget(
            Paragraph::new("todo\n\nResize to 35 × 12 or larger.\nq quit")
                .style(Style::default().fg(ACCENT))
                .wrap(Wrap { trim: true }),
            area,
        );
        return;
    }
    let show_input = app.vim.mode() == VimMode::Search
        || (app.vim.mode() == VimMode::Normal && !app.query.is_empty());
    let [header, _, body, message, input, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(u16::from(app.error)),
        Constraint::Length(u16::from(show_input)),
        Constraint::Length(1),
    ])
    .areas(workspace(area));
    draw_header(frame, app, header);
    draw_tasks(frame, app, body);
    if app.error {
        frame.render_widget(
            Paragraph::new(app.status.as_str()).style(Style::default().fg(ERROR)),
            message,
        );
    }
    draw_input(frame, app, input);
    draw_footer(frame, app, footer);
    if app.help {
        draw_help(frame, app);
    }
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let [brand, summary] =
        Layout::horizontal([Constraint::Length(6), Constraint::Min(0)]).areas(area);
    frame.render_widget(
        Paragraph::new("todo").style(Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        brand,
    );
    let done = app.todos.iter().filter(|todo| todo.done).count();
    let text = if app.todos.is_empty() {
        "0 tasks".into()
    } else {
        format!("{done}/{} done", app.todos.len())
    };
    frame.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Right)
            .style(Style::default().fg(MUTED)),
        summary,
    );
}

struct DisplayRow {
    index: Option<usize>,
    depth: usize,
}

fn draw_tasks(frame: &mut Frame, app: &mut App, area: Rect) {
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
        let position = (start..end)
            .find(|&row| {
                visible[row].depth == depth
                    && app.todos[visible[row].index.unwrap()].priority > app.input_priority
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
            let mut style = Style::default().fg(if selected {
                ACCENT
            } else if completed {
                MUTED
            } else {
                TEXT
            });
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
                Span::styled(branch, Style::default().fg(MUTED)),
                Span::styled(
                    if completed { "✓  " } else { "□  " },
                    Style::default().fg(if completed {
                        DONE
                    } else if selected {
                        ACCENT
                    } else {
                        TEXT
                    }),
                ),
                Span::styled(
                    format!("{:<4} ", priority.label()),
                    Style::default().fg(if completed {
                        MUTED
                    } else {
                        match priority {
                            Priority::High => Color::Red,
                            Priority::Mid => Color::Yellow,
                            Priority::Low => Color::Blue,
                        }
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
    let list = List::new(items)
        .highlight_symbol("› ")
        .highlight_style(Style::default().fg(ACCENT));
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

fn draw_input(frame: &mut Frame, app: &App, area: Rect) {
    let input = area;
    if input.height == 0 {
        return;
    }
    if app.vim.mode() == VimMode::Normal {
        if !app.query.is_empty() {
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled("/ ", Style::default().fg(ACCENT)),
                    Span::raw(app.query.as_str()),
                    Span::styled("  · Esc clear", Style::default().fg(MUTED)),
                ])),
                input,
            );
        }
        return;
    }
    if app.vim.mode() != VimMode::Search {
        return;
    }
    let label = "/ ";
    let prefix_width = unicode_width::UnicodeWidthStr::width(label) as u16;
    let (text, column) = app
        .editor
        .viewport(input.width.saturating_sub(prefix_width));
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(label, Style::default().fg(ACCENT)),
            Span::raw(text),
        ])),
        input,
    );
    if !app.help && input.width > prefix_width {
        frame.set_cursor_position((input.x + prefix_width + column, input.y));
    }
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let editing = app.vim.mode() != VimMode::Normal;
    let pending = app.vim.pending_label();
    let [mode, keys] = Layout::horizontal([Constraint::Length(14), Constraint::Min(0)]).areas(area);
    frame.render_widget(
        Paragraph::new(format!(
            "{}  {pending}",
            app.vim.mode().label().to_lowercase()
        ))
        .style(Style::default().fg(if editing || !pending.is_empty() {
            ACCENT
        } else {
            MUTED
        })),
        mode,
    );
    let hint = if pending.ends_with('p') {
        if keys.width < 22 {
            "h high m mid l low"
        } else {
            "h high · m mid · l low"
        }
    } else if editing {
        if area.width < 45 {
            "Enter save  Esc"
        } else if area.width < 60 {
            "Enter save · Esc cancel"
        } else if app.vim.mode() == VimMode::Insert && area.width >= 80 {
            "Enter save · Esc cancel · Ctrl-p priority"
        } else {
            "Enter save  ·  Esc cancel  ·  ←/→ move"
        }
    } else if area.width < 45 {
        "h/l tree  i  ?  q"
    } else if area.width < 75 {
        "i add · t priority · ? help"
    } else if area.width < 105 {
        "i add · t priority · x done · ? help · q quit"
    } else {
        "j/k move · h/l tree · i add · x done · t priority · ? help · q quit"
    };
    frame.render_widget(
        Paragraph::new(hint)
            .alignment(Alignment::Right)
            .style(Style::default().fg(MUTED)),
        keys,
    );
}

fn draw_help(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let width = 54.min(area.width.saturating_sub(2));
    let height = 25.min(area.height.saturating_sub(2));
    let popup = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, popup);
    let lines = [
        "j/k       next / previous task",
        "l         select / create child",
        "h         select parent",
        "gg / G    first / last task",
        "3j / 2k   repeat movement",
        "",
        "i/a/o     add sibling task",
        "e / cc    edit task",
        "t         cycle priority",
        "ph/pm/pl  high / mid / low",
        "ctrl-p    cycle while typing",
        "Space/x   toggle task + children",
        "dd / 3dd  delete task tree(s)",
        "u         undo deletion",
        "",
        "/         search all tasks",
        "Esc       cancel / clear search",
        "Enter     save / apply search",
        "←/→       move input cursor",
        "Home/End  start / end of input",
        "ctrl-←/→  move by word",
        "ctrl-w/u  delete word / clear",
        "ctrl-r    reload tasks",
        "q/ctrl-c  quit",
        "",
        "j/k scroll · Esc close",
    ];
    let block = Block::default()
        .title(" keys ")
        .title_style(Style::default().fg(ACCENT))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MUTED))
        .style(Style::default().bg(BG).fg(TEXT));
    frame.render_widget(
        Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<_>>())
            .block(block)
            .scroll((app.help_scroll, 0))
            .wrap(Wrap { trim: true }),
        popup,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use ratatui::{Terminal, backend::TestBackend};

    fn text_position(terminal: &Terminal<TestBackend>, text: &str) -> (u16, u16) {
        let buffer = terminal.backend().buffer();
        for y in 0..buffer.area.height {
            let line: String = (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect();
            if let Some(byte) = line.find(text) {
                return (line[..byte].chars().count() as u16, y);
            }
        }
        panic!("Text missing from terminal: {text}");
    }

    #[test]
    fn inline_editor_matches_tree_position_and_scrolls_into_view() {
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        let mut empty = App::new(Database::memory()).unwrap();
        empty.vim.set_mode(VimMode::Insert);
        empty.editor.insert("Draft task");
        terminal.draw(|frame| draw(frame, &mut empty)).unwrap();
        assert_eq!(text_position(&terminal, "Draft task"), (12, 3));

        let database = Database::memory();
        let parent = database.add("Parent", None, Priority::Mid).unwrap();
        let child = database.add("Child", Some(parent), Priority::Mid).unwrap();
        database
            .add("Grandchild", Some(child), Priority::Mid)
            .unwrap();
        let sibling = database
            .add("Sibling", Some(parent), Priority::Mid)
            .unwrap();
        database.add("Other root", None, Priority::Mid).unwrap();
        let mut app = App::new(database).unwrap();
        app.vim.set_mode(VimMode::Insert);
        app.editor.insert("Draft task");
        for (adding_parent, expected) in [
            (None, (12, 8)),
            (Some(parent), (17, 7)),
            (Some(child), (22, 6)),
            (Some(sibling), (22, 7)),
        ] {
            app.adding_parent = adding_parent;
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            assert_eq!(text_position(&terminal, "Draft task"), expected);
            let cursor = terminal.get_cursor_position().unwrap();
            assert_eq!((cursor.x, cursor.y), (expected.0 + 10, expected.1));
            assert_eq!(app.list.selected(), Some(0));
        }
        for (parent_id, priority, expected) in [
            (None, Priority::High, (12, 3)),
            (None, Priority::Low, (12, 8)),
            (Some(parent), Priority::High, (17, 4)),
            (Some(parent), Priority::Low, (17, 7)),
            (Some(child), Priority::High, (22, 5)),
        ] {
            app.adding_parent = parent_id;
            app.input_priority = priority;
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            assert_eq!(text_position(&terminal, "Draft task"), expected);
        }
        app.input_priority = Priority::Mid;
        // Editing replaces the existing title without adding another row.
        app.editing_id = Some(parent);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        assert_eq!(text_position(&terminal, "Draft task"), (12, 3));
        assert_eq!(text_position(&terminal, "Child").1, 4);

        let mut narrow = Terminal::new(TestBackend::new(35, 12)).unwrap();
        app.editing_id = None;
        app.adding_parent = Some(child);
        app.editor = crate::vim_motion::editor::Editor::new("界".repeat(40));
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        narrow.draw(|frame| draw(frame, &mut app)).unwrap();
        let cursor = narrow.get_cursor_position().unwrap();
        assert!(cursor.x < 34);
        assert_eq!(cursor.y, 6);

        let database = Database::memory();
        for row in 0..30 {
            database
                .add(&format!("Root {row}"), None, Priority::Mid)
                .unwrap();
        }
        let mut app = App::new(database).unwrap();
        app.vim.set_mode(VimMode::Insert);
        app.editor.insert("Draft task");
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        let (_, y) = text_position(&terminal, "Draft task");
        assert!(y < 18);
        assert_eq!(terminal.get_cursor_position().unwrap().y, y);
        assert!(app.list.offset() > 0);
        app.vim.set_mode(VimMode::Normal);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        assert_eq!(text_position(&terminal, "Root 0"), (12, 3));
    }

    #[test]
    fn renders_small_and_large_terminals_and_input_modes() {
        let mut empty = App::new(Database::memory()).unwrap();
        let database = Database::memory();
        let root = database.add("Parent task", None, Priority::Mid).unwrap();
        let child = database
            .add("Child task", Some(root), Priority::Mid)
            .unwrap();
        database
            .add("Grandchild task", Some(child), Priority::Mid)
            .unwrap();
        database
            .add("Sibling task", Some(root), Priority::Mid)
            .unwrap();
        database.add("Other root", None, Priority::Mid).unwrap();
        let mut app = App::new(database).unwrap();
        for (width, height) in [(20, 5), (35, 12), (80, 24), (120, 35), (192, 60)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal.draw(|frame| draw(frame, &mut empty)).unwrap();
            app.list.select(Some(0));
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            if width >= 80 {
                let buffer = terminal.backend().buffer();
                let rows: Vec<String> = (0..height)
                    .map(|y| (0..width).map(|x| buffer[(x, y)].symbol()).collect())
                    .collect();
                let positions: Vec<_> = [
                    "Parent task",
                    "Child task",
                    "Grandchild task",
                    "Sibling task",
                    "Other root",
                ]
                .iter()
                .map(|title| {
                    rows.iter()
                        .enumerate()
                        .find_map(|(y, line)| {
                            line.find(title)
                                .map(|byte| (y, line[..byte].chars().count()))
                        })
                        .unwrap()
                })
                .collect();
                // Padding is horizontal: every task occupies exactly one row.
                assert!(positions.windows(2).all(|pair| pair[1].0 == pair[0].0 + 1));
                assert_eq!(positions[1].1, positions[0].1 + 5);
                assert_eq!(positions[2].1, positions[1].1 + 5);
                assert_eq!(positions[3].1, positions[1].1);
                assert_eq!(positions[4].1, positions[0].1);
                assert!(rows[positions[1].0].contains("├──"));
                assert!(rows[positions[2].0].contains("│    └──"));
                assert!(rows[positions[3].0].contains("└──"));
                assert!(
                    rows.iter()
                        .all(|line| !line.contains("created") && !line.contains("───"))
                );
                assert!(
                    buffer.content.iter().all(|cell| {
                        cell.bg == Color::Reset && !matches!(cell.fg, Color::Rgb(..))
                    })
                );
            }
            app.list.select(Some(app.todos.len() - 1));
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            app.vim.set_mode(VimMode::Insert);
            app.editor.insert("A long task with Unicode 界 👩‍💻");
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            app.vim.set_mode(VimMode::Normal);
            app.error = true;
            app.status = "Could not save task".into();
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            app.error = false;
            app.help = true;
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            app.help = false;
        }
    }
}
