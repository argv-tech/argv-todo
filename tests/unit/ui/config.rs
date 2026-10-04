use crate::{
    app::{App, ConfigSetting},
    config::{Config, TaskView},
    db::Database,
    ui::{draw, tests::text_position},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Terminal, TerminalOptions, Viewport,
    backend::TestBackend,
    layout::Rect,
    style::{Color, Modifier},
};

fn app() -> App {
    let mut app = App::new(Database::memory()).unwrap();
    app.configuring = true;
    app.config = Some(Config {
        path: "/temporary/config.toml".into(),
        database_path: "db.sql".into(),
        active_database: "/temporary/db.sql".into(),
        database_override: false,
        task_view: TaskView::Normal,
        default_priority: crate::db::Priority::Mid,
        show_completed: true,
        sort_order: crate::config::SortOrder::Priority,
        show_hints: true,
    });
    app
}

fn press(app: &mut App, key: KeyCode) {
    app.handle_key(KeyEvent::new(key, KeyModifiers::NONE));
}

fn contains(terminal: &Terminal<TestBackend>, text: &str) -> bool {
    (0..terminal.backend().buffer().area.height)
        .any(|y| crate::ui::tests::terminal_row(terminal, y).contains(text))
}

#[test]
fn details_follow_keyboard_selection_and_saved_layout_changes() {
    let folder = std::env::temp_dir().join(format!(
        "argv-todo-setting-details-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut app = app();
    app.config = Some(Config::load(&folder.join("db.sql"), false).unwrap());
    let mut terminal = Terminal::new(TestBackend::new(120, 35)).unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "active database");
    assert!(!contains(&terminal, "layout preview"));

    press(&mut app, KeyCode::Char('j'));
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "task view · normal (live)");
    text_position(&terminal, "Full tree, including done tasks.");
    text_position(&terminal, "layout preview");
    assert!(!contains(&terminal, "active database"));
    assert!(!contains(&terminal, "configuration file"));

    press(&mut app, KeyCode::Enter);
    assert!(!app.error);
    assert_eq!(app.task_view(), TaskView::Split);
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let (x, y) = text_position(&terminal, "task view · split (live)");
    let cell = &terminal.backend().buffer()[(x + 12, y)];
    assert_eq!(cell.fg, Color::Cyan);
    assert!(cell.modifier.contains(Modifier::BOLD));
    let (x, y) = text_position(&terminal, "Parent (parent)");
    let cell = &terminal.backend().buffer()[(x, y)];
    assert_eq!(cell.fg, Color::DarkGray);
    assert!(cell.modifier.contains(Modifier::DIM));
    text_position(&terminal, "Todo / Completed · Tab switches");
    assert!(!contains(&terminal, "Full tree, including done tasks."));

    press(&mut app, KeyCode::Char('h'));
    assert!(!app.error);
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "task view · normal (live)");
    assert!(!contains(&terminal, "Parent (parent)"));

    for key in [KeyCode::Char('k'), KeyCode::Tab, KeyCode::BackTab] {
        press(&mut app, key);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        if app.config_setting == ConfigSetting::DatabasePath {
            text_position(&terminal, "active database");
            assert!(!contains(&terminal, "layout preview"));
        } else {
            text_position(&terminal, "task view · normal (live)");
            assert!(!contains(&terminal, "active database"));
        }
    }
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn task_view_details_survive_resize_and_nonzero_origins() {
    let mut app = app();
    app.config_setting = ConfigSetting::TaskView;
    let mut terminal = Terminal::new(TestBackend::new(120, 35)).unwrap();
    for view in [TaskView::Normal, TaskView::Split] {
        app.config.as_mut().unwrap().task_view = view;
        for (width, height) in [
            (120, 35),
            (80, 24),
            (74, 12),
            (73, 12),
            (35, 12),
            (1, 1),
            (0, 0),
        ] {
            terminal.backend_mut().resize(width, height);
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            if width < 35 {
                continue;
            }
            let (x, y) = text_position(&terminal, "task view · ");
            let (_, label_y) = text_position(&terminal, "database_path");
            assert_eq!(y == label_y, width >= 74);
            assert!(x < width);
            text_position(&terminal, view.label());
            text_position(&terminal, "(live)");
            text_position(
                &terminal,
                match view {
                    TaskView::Normal => "Full tree, including done tasks.",
                    TaskView::Split => "Todo / Completed · Tab switches",
                },
            );
            assert!(!contains(&terminal, "active database"));
            assert!(!contains(&terminal, "/temporary/"));
        }
    }

    let area = Rect::new(7, 4, 35, 12);
    let mut terminal = Terminal::with_options(
        TestBackend::new(55, 25),
        TerminalOptions {
            viewport: Viewport::Fixed(area),
        },
    )
    .unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let position = text_position(&terminal, "task view · split (live)");
    assert!(area.contains(position.into()));
    let buffer = terminal.backend().buffer();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            if !area.contains((x, y).into()) {
                assert_eq!(buffer[(x, y)].symbol(), " ");
            }
        }
    }
}

#[test]
fn new_settings_scroll_into_view_and_details_follow_selection_on_resize() {
    let mut app = app();
    app.config.as_mut().unwrap().task_view = TaskView::Split;
    let mut terminal = Terminal::new(TestBackend::new(120, 35)).unwrap();
    for setting in [
        ConfigSetting::DefaultPriority,
        ConfigSetting::ShowCompleted,
        ConfigSetting::SortOrder,
        ConfigSetting::ShowHints,
    ] {
        app.config_setting = setting;
        for (width, height) in [(120, 35), (80, 24), (74, 12), (73, 12), (35, 12)] {
            terminal.backend_mut().resize(width, height);
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            let (label_x, label_y) = text_position(&terminal, &format!("{}  ", setting.name()));
            let (details_x, details_y) = text_position(
                &terminal,
                &format!("{} · {}", setting.name(), setting.value(&app)),
            );
            if width >= 74 {
                assert!(details_x > label_x);
            } else {
                assert!(details_y > label_y);
            }
            let (value_x, value_y) = (label_x + setting.name().len() as u16 + 2, label_y);
            assert!(
                terminal.backend().buffer()[(value_x, value_y)]
                    .modifier
                    .contains(Modifier::UNDERLINED)
            );
            if setting == ConfigSetting::ShowCompleted {
                text_position(&terminal, "Always false in Split.");
            }
            assert!(!contains(&terminal, "active database"));
        }
    }
    let area = Rect::new(7, 4, 35, 12);
    let mut terminal = Terminal::with_options(
        TestBackend::new(55, 25),
        TerminalOptions {
            viewport: Viewport::Fixed(area),
        },
    )
    .unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    assert!(area.contains(text_position(&terminal, "show_hints · true").into()));
}

#[test]
fn hiding_hints_keeps_mode_errors_and_help_usable() {
    let mut app = app();
    app.config.as_mut().unwrap().show_hints = false;
    app.configuring = false;
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "normal");
    assert!(!contains(&terminal, "q quit"));
    assert!(!contains(&terminal, "? help"));
    app.configuring = true;
    app.config_setting = ConfigSetting::ShowHints;
    app.error = true;
    app.status = "Could not save config".into();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "show_hints · false");
    text_position(&terminal, "Could not save config");
    assert!(!contains(&terminal, "j/k setting"));
    app.help = true;
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "Esc / ? close");
}

#[test]
fn manual_sort_places_inline_sibling_drafts_at_the_saved_position() {
    use crate::{config::SortOrder, db::Priority, vim_motion::InputTarget};
    let database = Database::memory();
    let first = database.add("Low root", None, Priority::Low).unwrap();
    database.add("Child", Some(first), Priority::Low).unwrap();
    let second = database.add("High root", None, Priority::High).unwrap();
    let config = app().config;
    let mut app = App::new(database).unwrap();
    app.config = config;
    app.config.as_mut().unwrap().sort_order = SortOrder::Manual;
    app.adding_relative = Some((second, true));
    app.input_priority = Priority::Mid;
    app.vim.begin_input(InputTarget::Task);
    app.editor.insert("Draft sibling");
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let child_y = text_position(&terminal, "Child").1;
    let draft_y = text_position(&terminal, "Draft sibling").1;
    let second_y = text_position(&terminal, "High root").1;
    assert_eq!(draft_y, child_y + 1);
    assert_eq!(second_y, draft_y + 1);
}
