use super::config::CONFIG_LOGO;
use super::*;
use crate::db::Database;
use crate::db::Priority;
use crate::{
    app::App,
    vim_motion::{InputTarget, VimMode},
};
use ratatui::style::{Color, Modifier, Style};
use ratatui::{Terminal, TerminalOptions, Viewport, backend::TestBackend, layout::Rect};
use unicode_width::UnicodeWidthStr;

pub(super) fn text_position(terminal: &Terminal<TestBackend>, text: &str) -> (u16, u16) {
    let buffer = terminal.backend().buffer();
    for y in 0..buffer.area.height {
        let mut line = String::new();
        let mut columns = Vec::new();
        for x in 0..buffer.area.width {
            columns.push((line.len(), x));
            line.push_str(buffer[(x, y)].symbol());
        }
        if let Some(byte) = line.find(text) {
            return (
                columns
                    .iter()
                    .find(|(offset, _)| *offset == byte)
                    .unwrap()
                    .1,
                y,
            );
        }
    }
    panic!("Text missing from terminal: {text}");
}

pub(super) fn terminal_row(terminal: &Terminal<TestBackend>, y: u16) -> String {
    let buffer = terminal.backend().buffer();
    (0..buffer.area.width)
        .map(|x| buffer[(x, y)].symbol())
        .collect()
}

#[test]
fn header_keeps_version_and_completion_count_visible_on_resize() {
    let mut app = App::new(Database::memory()).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let version = concat!("v", env!("CARGO_PKG_VERSION"));
    for (width, height) in [(80, 24), (35, 12), (1, 1), (0, 0), (120, 35)] {
        terminal.backend_mut().resize(width, height);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        if width < 35 {
            continue;
        }
        let (brand_x, brand_y) = text_position(&terminal, "argv-todo");
        let (version_x, version_y) = text_position(&terminal, version);
        let (summary_x, summary_y) = text_position(&terminal, "0 tasks");
        assert_eq!(version_x, brand_x + 10);
        assert_eq!(version_y, brand_y);
        assert_eq!(summary_y, brand_y);
        assert!(summary_x > version_x + version.width() as u16);
        let buffer = terminal.backend().buffer();
        assert!(buffer[(brand_x, brand_y)].modifier.contains(Modifier::BOLD));
        for x in version_x..version_x + version.width() as u16 {
            assert_eq!(buffer[(x, version_y)].fg, Color::Red);
            assert!(!buffer[(x, version_y)].modifier.contains(Modifier::BOLD));
        }
    }

    let mut database = Database::memory();
    let task = database.add("Task", None, Priority::Mid).unwrap();
    database.toggle(task).unwrap();
    let mut app = App::new(database).unwrap();
    let area = Rect::new(7, 4, 35, 12);
    let mut terminal = Terminal::with_options(
        TestBackend::new(55, 25),
        TerminalOptions {
            viewport: Viewport::Fixed(area),
        },
    )
    .unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    assert_eq!(text_position(&terminal, version), (18, 5));
    let (summary_x, summary_y) = text_position(&terminal, "1/1 done");
    assert_eq!(summary_y, 5);
    assert_eq!(summary_x + 8, area.right() - 1);
}

#[test]
fn database_preview_tracks_drafts_and_saved_paths() {
    let folder = std::env::temp_dir().join("argv-todo-preview");
    let active = folder.join("db.sql");
    let mut app = App::new(Database::memory()).unwrap();
    app.configuring = true;
    app.config = Some(crate::config::Config {
        path: folder.join("config.toml"),
        database_path: "db.sql".into(),
        active_database: active.clone(),
        database_override: false,
        task_view: crate::config::TaskView::Normal,
        default_priority: crate::db::Priority::Mid,
        show_completed: true,
        sort_order: crate::config::SortOrder::Priority,
        show_hints: true,
    });
    let mut terminal = Terminal::new(TestBackend::new(120, 35)).unwrap();
    let absolute = folder.join("absolute.sql").display().to_string();
    for (saved, draft, expected) in [
        ("db.sql", None, None),
        ("db.sql", Some("./db.sql"), None),
        (
            "db.sql",
            Some("storage/tasks.sql"),
            Some(folder.join("storage/tasks.sql")),
        ),
        ("db.sql", Some(""), None),
        ("db.sql", Some("   "), None),
        (
            "db.sql",
            Some(absolute.as_str()),
            Some(folder.join("absolute.sql")),
        ),
        ("db.sql", None, None),
        ("saved.sql", None, Some(folder.join("saved.sql"))),
        ("saved.sql", Some("db.sql"), None),
    ] {
        app.config.as_mut().unwrap().database_path = saved.into();
        if let Some(draft) = draft {
            app.vim.begin_input(InputTarget::DatabasePath);
            app.editor.reset(draft.into());
        } else {
            app.vim.end_input();
        }
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        if let Some(expected) = expected {
            let (x, y) = text_position(&terminal, "database preview (after restart)");
            let (_, active_y) = text_position(&terminal, "active database");
            let (_, config_y) = text_position(&terminal, "configuration file");
            assert!(active_y < y && y < config_y);
            assert!(terminal_row(&terminal, y + 1).contains(&expected.display().to_string()));
            assert!(
                terminal.backend().buffer()[(x, y)]
                    .modifier
                    .contains(Modifier::BOLD)
            );
        } else {
            assert!((0..35).all(|y| !terminal_row(&terminal, y).contains("database preview")));
        }
        let config = app.config.as_ref().unwrap();
        assert_eq!(config.active_database, active);
        assert_eq!(config.database_path, saved);
    }
}

#[test]
fn database_preview_remains_visible_on_resize() {
    let mut app = App::new(Database::memory()).unwrap();
    app.configuring = true;
    app.config = Some(crate::config::Config {
        path: "/temporary/config.toml".into(),
        database_path: "storage/界é👩‍💻tasks.sql".into(),
        active_database: "/temporary/db.sql".into(),
        database_override: true,
        task_view: crate::config::TaskView::Normal,
        default_priority: crate::db::Priority::Mid,
        show_completed: true,
        sort_order: crate::config::SortOrder::Priority,
        show_hints: true,
    });
    let mut terminal = Terminal::new(TestBackend::new(120, 35)).unwrap();
    for (width, height) in [(120, 35), (100, 17), (74, 12), (73, 12), (35, 12)] {
        terminal.backend_mut().resize(width, height);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        let preview_y = text_position(
            &terminal,
            if height >= 17 {
                "database preview (after restart)"
            } else {
                "preview (after restart) "
            },
        )
        .1;
        let active_y = text_position(
            &terminal,
            if width >= 74 {
                "active database"
            } else {
                "using   "
            },
        )
        .1;
        assert!(preview_y > active_y);
        assert!(terminal_row(&terminal, preview_y + u16::from(height >= 17)).contains("/temp"));
        if width >= 74 {
            text_position(&terminal, "Current launch uses --db.");
        }
    }
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
        task_view: crate::config::TaskView::Normal,
        default_priority: crate::db::Priority::Mid,
        show_completed: true,
        sort_order: crate::config::SortOrder::Priority,
        show_hints: true,
    });
    for (width, height) in [(20, 5), (35, 12), (60, 16), (80, 24), (120, 35)] {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        if width < 35 {
            continue;
        }
        text_position(&terminal, "configuration");
        text_position(&terminal, "database_path");
        text_position(&terminal, "task_view");
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
        app.vim.begin_input(InputTarget::DatabasePath);
        app.editor = crate::vim_motion::Editor::new("界".repeat(80));
        app.error = true;
        app.status = "Invalid database path".into();
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        let cursor = terminal.get_cursor_position().unwrap();
        assert!(cursor.x < width && cursor.y < height);
        let label_y = text_position(&terminal, "database_path").1;
        assert_eq!(cursor.y, label_y + 1);
        text_position(&terminal, "Invalid database path");
        text_position(&terminal, "Esc normal");
        app.vim.end_input();
        app.error = false;
        app.status.clear();
    }
}

#[test]
fn inline_editor_matches_tree_position_and_scrolls_into_view() {
    let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
    let mut empty = App::new(Database::memory()).unwrap();
    empty.vim.begin_input(InputTarget::Task);
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
    app.vim.begin_input(InputTarget::Task);
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
    app.vim.begin_input(InputTarget::Task);
    app.editor.insert("Draft task");
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let (_, y) = text_position(&terminal, "Draft task");
    assert!(y < 18);
    assert_eq!(terminal.get_cursor_position().unwrap().y, y);
    assert!(app.list.offset() > 0);
    app.vim.end_input();
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
        app.vim.begin_input(InputTarget::Task);
        app.editor.insert("A long task with Unicode 界 👩‍💻");
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        app.vim.end_input();
        app.error = true;
        app.status = "Could not save task".into();
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        app.error = false;
        app.help = true;
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        app.help = false;
    }
}

#[test]
fn long_titles_keep_child_counts_and_semantic_styles() {
    let database = Database::memory();
    let parent = database
        .add(&"Long parent 界 é 👩‍💻 ".repeat(10), None, Priority::High)
        .unwrap();
    database.add("Child", Some(parent), Priority::Low).unwrap();
    let mut app = App::new(database).unwrap();
    app.todos[0].done = true;
    app.todos[1].done = true;
    let mut terminal = Terminal::new(TestBackend::new(35, 12)).unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let (x, y) = text_position(&terminal, "Long parent");
    let row = terminal_row(&terminal, y);
    assert!(row.contains('…'));
    text_position(&terminal, "1/1");
    let buffer = terminal.backend().buffer();
    let title = &buffer[(x, y)];
    assert!(
        title
            .modifier
            .contains(Modifier::BOLD | Modifier::UNDERLINED | Modifier::CROSSED_OUT)
    );
    assert_eq!(title.fg, Color::Reset);
    let (priority_x, _) = text_position(&terminal, "high");
    assert_eq!(buffer[(priority_x, y)].fg, Color::Red);
    let (child_x, child_y) = text_position(&terminal, "Child");
    assert!(
        buffer[(child_x, child_y)]
            .modifier
            .contains(Modifier::CROSSED_OUT)
    );
    assert!(
        !buffer[(child_x, child_y)]
            .modifier
            .contains(Modifier::UNDERLINED)
    );
    assert_eq!(app.list.selected(), Some(0));

    app.list.select(Some(1));
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    assert!(
        !terminal.backend().buffer()[(x, y)]
            .modifier
            .contains(Modifier::UNDERLINED)
    );
    assert!(
        terminal.backend().buffer()[(child_x, child_y)]
            .modifier
            .contains(Modifier::UNDERLINED)
    );
}

#[test]
fn truncation_preserves_graphemes_and_display_width() {
    for (text, width, expected) in [
        ("hello", 0, ""),
        ("hello", 1, "…"),
        ("hello", 5, "hello"),
        ("界界界", 4, "界…"),
        ("éxyz", 2, "é…"),
        ("👩‍💻abc", 3, "👩‍💻…"),
    ] {
        let actual = text::truncate(text, width);
        assert_eq!(actual, expected);
        assert!(actual.width() <= width);
    }
}

#[test]
fn footer_hints_fit_and_describe_the_active_mode() {
    let mut app = App::new(Database::memory()).unwrap();
    for width in [35, 44, 45, 59, 60, 74, 75, 79, 80, 104, 105, 120] {
        let mut terminal = Terminal::new(TestBackend::new(width, 12)).unwrap();
        for (target, expected) in [
            (None, "q"),
            (Some(InputTarget::Search), "Enter apply"),
            (Some(InputTarget::Task), "Enter save"),
        ] {
            if let Some(target) = target {
                app.vim.begin_input(target);
            } else {
                app.vim.end_input();
            }
            let mode = app.vim.mode();
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            let row = terminal_row(&terminal, 10);
            assert!(row.contains(expected), "{width}: {row}");
            assert!(row.contains("Esc"), "{width}: {row}");
            assert!(row.contains(mode.label().to_lowercase().as_str()));
            assert!(
                row.trim_end().ends_with(if mode == VimMode::Normal {
                    "q"
                } else {
                    "cancel"
                }) || row.trim_end().ends_with("Esc")
                    || row.trim_end().ends_with("normal")
                    || row.trim_end().ends_with("move")
                    || row.trim_end().ends_with("priority")
                    || row.trim_end().ends_with("quit"),
                "Clipped hint at {width}: {row}"
            );
        }
    }
    app.vim.end_input();
    app.query = "A very long applied search 界 ".repeat(8);
    let mut terminal = Terminal::new(TestBackend::new(35, 12)).unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "Esc clear");
    assert!(terminal_row(&terminal, 9).contains('…'));
    assert!(!terminal.backend().cursor_visible());
}

#[test]
fn help_keeps_close_hint_visible_and_clears_underlying_editor() {
    let mut app = App::new(Database::memory()).unwrap();
    app.vim.begin_input(InputTarget::Task);
    app.editor.insert("Draft underneath help");
    let mut terminal = Terminal::new(TestBackend::new(35, 12)).unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    assert!(terminal.backend().cursor_visible());
    app.help = true;
    for key in ['g', 'g', 'j', 'G'] {
        app.handle_key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(key),
            crossterm::event::KeyModifiers::NONE,
        ));
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        let (x, y) = text_position(&terminal, "Esc / ? close");
        assert_eq!(y, 10);
        assert_eq!(terminal.backend().buffer()[(x, y)].fg, Color::Cyan);
        assert!(!terminal.backend().cursor_visible());
        assert!(!(0..12).any(|y| terminal_row(&terminal, y).contains("Draft underneath")));
    }
    text_position(&terminal, "swap case in Visual");
    app.help = false;
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "Draft underneath help");
    assert!(terminal.backend().cursor_visible());
}

#[test]
fn resize_and_nonzero_viewport_keep_editor_inside_workspace() {
    let mut app = App::new(Database::memory()).unwrap();
    app.vim.begin_input(InputTarget::Search);
    app.editor.insert(&"é界👩‍💻".repeat(30));
    let area = Rect::new(7, 4, 35, 12);
    let mut terminal = Terminal::with_options(
        TestBackend::new(55, 25),
        TerminalOptions {
            viewport: Viewport::Fixed(area),
        },
    )
    .unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let cursor = terminal.backend().cursor_position();
    assert!(area.contains(cursor));
    assert_eq!(cursor.y, 13);
    let buffer = terminal.backend().buffer();
    for y in 0..25 {
        for x in 0..55 {
            if !area.contains((x, y).into()) {
                assert_eq!(buffer[(x, y)].symbol(), " ");
            }
        }
    }

    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    for (width, height) in [(35, 12), (1, 1), (0, 0), (34, 12), (35, 11), (120, 35)] {
        terminal.backend_mut().resize(width, height);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        assert_eq!(
            terminal.backend().buffer().area,
            Rect::new(0, 0, width, height)
        );
        if width >= 35 && height >= 12 {
            let cursor = terminal.backend().cursor_position();
            assert!(cursor.x < width - 1 && cursor.y < height - 1);
            assert!(terminal.backend().cursor_visible());
        } else {
            assert!(!terminal.backend().cursor_visible());
        }
    }
}

#[test]
fn shared_editor_tolerates_empty_and_tiny_areas() {
    let editor = crate::vim_motion::Editor::new("界é👩‍💻".into());
    let mut terminal = Terminal::new(TestBackend::new(10, 5)).unwrap();
    for area in [
        Rect::new(7, 4, 0, 0),
        Rect::new(7, 4, 1, 1),
        Rect::new(7, 4, 2, 1),
        Rect::new(7, 4, 3, 1),
    ] {
        terminal
            .draw(|frame| input::draw_editor(frame, &editor, "/ ", Style::default(), area, true))
            .unwrap();
        assert_eq!(terminal.backend().cursor_visible(), area.width > 2);
        if terminal.backend().cursor_visible() {
            assert!(area.contains(terminal.backend().cursor_position()));
        }
    }
}

#[test]
fn configuration_splits_editor_and_paths_then_stacks_on_resize() {
    let mut app = App::new(Database::memory()).unwrap();
    app.configuring = true;
    app.config = Some(crate::config::Config {
        path: "/temporary/config.toml".into(),
        database_path: "db.sql".into(),
        active_database: "/temporary/tasks.sql".into(),
        database_override: true,
        task_view: crate::config::TaskView::Normal,
        default_priority: crate::db::Priority::Mid,
        show_completed: true,
        sort_order: crate::config::SortOrder::Priority,
        show_hints: true,
    });
    let mut terminal = Terminal::new(TestBackend::new(100, 17)).unwrap();
    for (width, height) in [(100, 17), (74, 12), (73, 12), (35, 12), (120, 35)] {
        terminal.backend_mut().resize(width, height);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        let (label_x, label_y) = text_position(&terminal, "database_path");
        let (_, editor_y) = text_position(&terminal, "db.sql");
        assert_eq!(editor_y, label_y + 1);
        if width >= 74 {
            let (paths_x, paths_y) = text_position(&terminal, "active database");
            assert!(paths_x > label_x + 25);
            assert_eq!(paths_y, label_y);
            text_position(&terminal, "configuration file");
            let (database_x, database_y) = text_position(&terminal, "/temporary/tasks.sql");
            assert_eq!(database_x, paths_x);
            assert_eq!(database_y, paths_y + 1);
            assert_eq!(
                terminal.backend().buffer()[(database_x, database_y)].fg,
                Color::Reset
            );
            let buffer = terminal.backend().buffer();
            assert!((label_x + 25..paths_x).any(|x| buffer[(x, paths_y)].symbol() == "│"));
            text_position(&terminal, "Current launch uses --db.");
        } else {
            let (paths_x, paths_y) = text_position(&terminal, "config  ");
            assert_eq!(paths_x, label_x);
            assert!(paths_y > editor_y);
        }
        app.vim.begin_input(InputTarget::DatabasePath);
        app.editor = crate::vim_motion::Editor::new("界é👩‍💻".repeat(40));
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        let cursor = terminal.backend().cursor_position();
        assert_eq!(cursor.y, editor_y);
        assert!(cursor.x < if width >= 74 { width / 2 } else { width - 1 });
        app.vim.end_input();
    }
}

fn field_keys(app: &mut App, text: &str) {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    for character in text.chars() {
        let code = if character == '\u{1b}' {
            KeyCode::Esc
        } else {
            KeyCode::Char(character)
        };
        if let Some(action) = app.vim.handle(KeyEvent::new(code, KeyModifiers::NONE)) {
            app.editor.apply(action);
            app.vim.set_mode(app.editor.mode());
        }
    }
}

#[test]
fn every_field_keeps_normal_cursor_and_renders_visual_selection_styles_on_resize() {
    for target in [
        InputTarget::Task,
        InputTarget::Search,
        InputTarget::DatabasePath,
    ] {
        let mut app = App::new(Database::memory()).unwrap();
        app.configuring = target == InputTarget::DatabasePath;
        app.vim.begin_input(target);
        app.editor = crate::vim_motion::Editor::new("é界👩‍💻 tail".into());
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        field_keys(&mut app, "\u{1b}0");
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        let (x, y) = text_position(&terminal, "é");
        assert_eq!(terminal.backend().cursor_position(), (x, y).into());
        assert!(terminal.backend().cursor_visible());
        text_position(&terminal, "normal");

        field_keys(&mut app, "v2l");
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        let buffer = terminal.backend().buffer();
        for column in [x, x + 1, x + 3] {
            assert!(buffer[(column, y)].modifier.contains(Modifier::REVERSED));
        }
        assert!(!buffer[(x + 5, y)].modifier.contains(Modifier::REVERSED));
        assert_eq!(terminal.backend().cursor_position(), (x + 3, y).into());
        text_position(&terminal, "visual");

        for (width, height) in [(35, 12), (1, 1), (0, 0), (120, 35)] {
            terminal.backend_mut().resize(width, height);
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            if width >= 35 {
                assert!(terminal.backend().cursor_visible());
                assert!(
                    terminal
                        .backend()
                        .buffer()
                        .area
                        .contains(terminal.backend().cursor_position())
                );
                assert!(
                    terminal
                        .backend()
                        .buffer()
                        .content
                        .iter()
                        .any(|cell| cell.modifier.contains(Modifier::REVERSED))
                );
            } else {
                assert!(!terminal.backend().cursor_visible());
            }
        }
        field_keys(&mut app, "\u{1b}");
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        assert!(
            terminal
                .backend()
                .buffer()
                .content
                .iter()
                .all(|cell| !cell.modifier.contains(Modifier::REVERSED))
        );
        assert_eq!(app.vim.input(), Some(target));
    }
}

#[test]
fn visual_line_styles_cover_only_field_text_and_pending_commands_stay_visible() {
    let mut app = App::new(Database::memory()).unwrap();
    app.vim.begin_input(InputTarget::Search);
    app.editor = crate::vim_motion::Editor::new("first second".into());
    field_keys(&mut app, "\u{1b}V");
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let (x, y) = text_position(&terminal, "first second");
    let buffer = terminal.backend().buffer();
    assert!(!buffer[(x - 1, y)].modifier.contains(Modifier::REVERSED));
    for column in x..x + 12 {
        assert!(buffer[(column, y)].modifier.contains(Modifier::REVERSED));
    }
    assert!(!buffer[(x + 12, y)].modifier.contains(Modifier::REVERSED));
    text_position(&terminal, "v-line");
    field_keys(&mut app, "\u{1b}2d3i");
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "normal  2d3i");
}
