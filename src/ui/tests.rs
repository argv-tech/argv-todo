use super::config::CONFIG_LOGO;
use super::*;
use crate::db::Database;
use crate::db::Priority;
use ratatui::style::Color;
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
fn config_view_shows_logo_when_it_fits_and_keeps_editor_visible() {
    let mut app = App::new(Database::memory()).unwrap();
    app.configuring = true;
    app.config = Some(crate::config::Config {
        path: "/temporary/config.toml".into(),
        database_path: "db.sql".into(),
        active_database: "/temporary/tasks.sql".into(),
        database_override: true,
    });
    for (width, height) in [(20, 5), (35, 12), (60, 16), (80, 24), (120, 35)] {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        if width < 35 {
            continue;
        }
        text_position(&terminal, "configuration");
        text_position(&terminal, "database_path");
        text_position(&terminal, "db.sql");
        text_position(&terminal, "Esc");
        if width >= 80 {
            let first_row = text_position(&terminal, CONFIG_LOGO[0].trim_end()).1;
            for (row, line) in CONFIG_LOGO.iter().enumerate() {
                assert_eq!(
                    text_position(&terminal, line.trim_end()).1,
                    first_row + row as u16
                );
            }
            text_position(&terminal, "Current launch uses --db.");
        }
        app.vim.set_mode(VimMode::Insert);
        app.editor = crate::vim_motion::Editor::new("界".repeat(80));
        app.error = true;
        app.status = "Invalid database path".into();
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        let cursor = terminal.get_cursor_position().unwrap();
        assert!(cursor.x < width && cursor.y < height);
        let label_y = text_position(&terminal, "database_path").1;
        assert_eq!(cursor.y, label_y + 1);
        text_position(&terminal, "Invalid database path");
        text_position(&terminal, "Esc cancel");
        app.vim.set_mode(VimMode::Normal);
        app.error = false;
        app.status.clear();
    }
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
    for (id, above, parent_id, expected) in [
        (parent, true, None, (12, 3)),
        (parent, false, None, (12, 7)),
        (child, true, Some(parent), (17, 4)),
        (child, false, Some(parent), (17, 6)),
        (sibling, false, Some(parent), (17, 7)),
    ] {
        app.adding_parent = parent_id;
        app.adding_relative = Some((id, above));
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        assert_eq!(text_position(&terminal, "Draft task"), expected);
    }
    app.adding_relative = None;
    // Editing replaces the existing title without adding another row.
    app.editing_id = Some(parent);
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    assert_eq!(text_position(&terminal, "Draft task"), (12, 3));
    assert_eq!(text_position(&terminal, "Child").1, 4);

    let mut narrow = Terminal::new(TestBackend::new(35, 12)).unwrap();
    app.editing_id = None;
    app.adding_parent = Some(child);
    app.editor = crate::vim_motion::Editor::new("界".repeat(40));
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
                buffer
                    .content
                    .iter()
                    .all(|cell| { cell.bg == Color::Reset && !matches!(cell.fg, Color::Rgb(..)) })
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
