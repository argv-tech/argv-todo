use crate::{
    app::{App, Filter},
    vim_motion::vim_mode::VimMode,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

const BG: Color = Color::Rgb(19, 22, 28);
const TEXT: Color = Color::Rgb(226, 230, 237);
const MUTED: Color = Color::Rgb(150, 161, 179);
const ACCENT: Color = Color::Rgb(143, 216, 186);
const RULE: Color = Color::Rgb(67, 78, 95);
const SELECTED: Color = Color::Rgb(30, 45, 44);
const ERROR: Color = Color::Rgb(244, 146, 146);

fn workspace(area: Rect) -> Rect {
    let margin = if area.width >= 60 { 2 } else { 1 };
    Rect::new(
        area.x + margin,
        area.y + 1,
        area.width.saturating_sub(margin * 2),
        area.height.saturating_sub(2),
    )
}

fn rule(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new("─".repeat(area.width as usize)).style(Style::default().fg(RULE)),
        area,
    );
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
    let [
        header,
        _gap,
        tabs,
        separator,
        body,
        metadata,
        message,
        input,
        footer,
    ] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(u16::from(area.height >= 16)),
        Constraint::Length(1),
        Constraint::Length(if area.height >= 16 { 2 } else { 1 }),
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
    ])
    .areas(workspace(area));
    draw_header(frame, app, header);
    draw_tabs(frame, app, tabs);
    rule(
        frame,
        Rect {
            height: 1,
            ..separator
        },
    );
    draw_tasks(frame, app, body);
    if let Some(todo) = app.selected_todo() {
        let text = if metadata.width >= 65 {
            format!(
                "  #{}  ·  {}  ·  created {} UTC",
                todo.id,
                if todo.done { "completed" } else { "active" },
                todo.created_at
            )
        } else {
            format!(
                "  #{}  ·  {}",
                todo.id,
                if todo.done { "completed" } else { "active" }
            )
        };
        frame.render_widget(
            Paragraph::new(text).style(Style::default().fg(MUTED)),
            metadata,
        );
    }
    frame.render_widget(
        Paragraph::new(app.status.as_str()).style(Style::default().fg(if app.error {
            ERROR
        } else {
            MUTED
        })),
        message,
    );
    draw_input(frame, app, input);
    draw_footer(frame, app, footer);
    if app.help {
        draw_help(frame, app);
    }
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let [brand, summary] =
        Layout::horizontal([Constraint::Length(12), Constraint::Min(0)]).areas(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                "todo",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" / rs", Style::default().fg(MUTED)),
        ])),
        brand,
    );
    let done = app.todos.iter().filter(|todo| todo.done).count();
    let text = if app.todos.is_empty() {
        "0 tasks".into()
    } else {
        format!("{} open  /  {done} done", app.todos.len() - done)
    };
    frame.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Right)
            .style(Style::default().fg(MUTED)),
        summary,
    );
}

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let done = app.todos.iter().filter(|todo| todo.done).count();
    let mut spans = Vec::new();
    for (filter, count) in [
        (Filter::All, app.todos.len()),
        (Filter::Active, app.todos.len() - done),
        (Filter::Done, done),
    ] {
        let selected = app.filter == filter;
        let style = if selected {
            Style::default()
                .fg(ACCENT)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(MUTED)
        };
        spans.push(Span::styled(format!("{} {count}", filter.label()), style));
        spans.push(Span::raw("    "));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_tasks(frame: &mut Frame, app: &mut App, area: Rect) {
    let visible = app.visible_indices();
    if visible.is_empty() {
        let query = if app.vim.mode() == VimMode::Search {
            app.editor.text()
        } else {
            &app.query
        };
        let (title, hint) = if !query.is_empty() {
            ("No matching tasks.", "Esc  clear search")
        } else if app.todos.is_empty() {
            ("Nothing on your list.", "i  add your first task")
        } else if app.filter == Filter::Active {
            ("All caught up.", "i  add a task  ·  h/l  change filter")
        } else {
            ("No completed tasks yet.", "h/l  change filter")
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
    let items: Vec<_> = visible
        .iter()
        .enumerate()
        .map(|(row, &index)| {
            let todo = &app.todos[index];
            let selected = app.list.selected() == Some(row);
            let style = if todo.done {
                Style::default().fg(MUTED)
            } else {
                Style::default().fg(TEXT)
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:>2}  ", row + 1),
                    Style::default().fg(if selected { ACCENT } else { MUTED }),
                ),
                Span::styled(
                    if todo.done { "✓  " } else { "○  " },
                    Style::default().fg(if todo.done || selected { ACCENT } else { MUTED }),
                ),
                Span::styled(todo.title.as_str(), style),
            ]))
        })
        .collect();
    let list = List::new(items)
        .highlight_symbol("▎ ")
        .highlight_style(Style::default().bg(SELECTED).fg(ACCENT));
    frame.render_stateful_widget(list, area, &mut app.list);
}

fn draw_input(frame: &mut Frame, app: &App, area: Rect) {
    rule(frame, Rect { height: 1, ..area });
    let input = Rect::new(
        area.x,
        area.y + 1,
        area.width,
        area.height.saturating_sub(1),
    );
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
    let label = match app.vim.mode() {
        VimMode::Search => "/ ",
        VimMode::Insert if app.editing_id.is_some() => "edit › ",
        _ => "add › ",
    };
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
        Paragraph::new(format!("{}  {pending}", app.vim.mode().label()))
            .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        mode,
    );
    let hint = if editing {
        if area.width < 45 {
            "Enter save  Esc"
        } else if area.width < 60 {
            "Enter save · Esc cancel"
        } else {
            "Enter save  ·  Esc cancel  ·  ←/→ move"
        }
    } else if area.width < 45 {
        "i add  ? help  q"
    } else if area.width < 60 {
        "i add · ? help · q quit"
    } else {
        "hjkl move  ·  i add  ·  x done  ·  ? help  ·  q quit"
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
        "h/l       change filter",
        "gg / G    first / last task",
        "3j / 2k   repeat movement",
        "",
        "i/a/o     add task",
        "e / cc    edit task",
        "Space/x   toggle completion",
        "dd / 3dd  delete task(s)",
        "u         undo deletion",
        "",
        "/         search titles",
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
        .border_style(Style::default().fg(RULE))
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

    #[test]
    fn renders_small_and_large_terminals_and_input_modes() {
        let mut app = App::new(Database::memory()).unwrap();
        for (width, height) in [(20, 5), (35, 12), (80, 24), (120, 35), (192, 60)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            app.vim.set_mode(VimMode::Insert);
            app.editor.insert("A long task with Unicode 界 👩‍💻");
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            app.vim.set_mode(VimMode::Normal);
            app.help = true;
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            app.help = false;
        }
    }
}
