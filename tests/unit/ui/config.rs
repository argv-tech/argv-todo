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
        completed_delete: crate::db::DeleteMode::Soft,
        notifications: true,
        notify_reminder_1_minutes: 60,
        notify_reminder_2_minutes: 1440,
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
    for view in [TaskView::Normal, TaskView::Split, TaskView::Projects] {
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
            let (label_x, label_y) = text_position(&terminal, "task_view  ");
            if width >= 74 {
                assert!(x > label_x);
                assert!(y <= label_y);
            } else {
                assert!(y > label_y);
            }
            assert!(x < width);
            text_position(&terminal, view.label());
            text_position(&terminal, "(live)");
            text_position(
                &terminal,
                match view {
                    TaskView::Normal => "Full tree, including done tasks.",
                    TaskView::Split => "Todo / Completed · Tab switches",
                    TaskView::Projects => "Project tabs · Tab switches",
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
    let position = text_position(&terminal, "task view · projects (live)");
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
        ConfigSetting::CompletedDelete,
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
fn settings_are_grouped_and_keyboard_navigation_skips_headings() {
    let mut app = app();
    let mut terminal = Terminal::new(TestBackend::new(120, 35)).unwrap();
    let groups: &[(&str, &[ConfigSetting])] = &[
        ("STORAGE", &[ConfigSetting::DatabasePath]),
        (
            "DISPLAY",
            &[
                ConfigSetting::TaskView,
                ConfigSetting::ShowCompleted,
                ConfigSetting::ShowHints,
            ],
        ),
        (
            "TASK BEHAVIOR",
            &[
                ConfigSetting::DefaultPriority,
                ConfigSetting::SortOrder,
                ConfigSetting::CompletedDelete,
            ],
        ),
        (
            "NOTIFICATIONS",
            &[
                ConfigSetting::Notifications,
                ConfigSetting::NotifyReminder1Minutes,
                ConfigSetting::NotifyReminder2Minutes,
            ],
        ),
    ];
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let mut previous_y = 0;
    for &(heading, settings) in groups {
        let (x, heading_y) = text_position(&terminal, heading);
        assert!(heading_y > previous_y);
        let cell = &terminal.backend().buffer()[(x, heading_y)];
        assert_eq!(cell.fg, Color::Cyan);
        assert!(cell.modifier.contains(Modifier::BOLD));
        previous_y = heading_y;
        for &setting in settings {
            assert_eq!(app.config_setting, setting);
            let (_, setting_y) = text_position(&terminal, setting.name());
            assert!(setting_y > previous_y);
            previous_y = setting_y;
            press(&mut app, KeyCode::Char('j'));
        }
    }
    assert_eq!(app.config_setting, ConfigSetting::NotifyReminder2Minutes);
    for &(_, settings) in groups.iter().rev() {
        for &setting in settings.iter().rev() {
            assert_eq!(app.config_setting, setting);
            press(&mut app, KeyCode::Char('k'));
        }
    }
    assert_eq!(app.config_setting, ConfigSetting::DatabasePath);
}

#[test]
fn short_forms_keep_the_selected_category_and_setting_visible() {
    let mut app = app();
    let mut terminal = Terminal::new(TestBackend::new(45, 12)).unwrap();
    for setting in ConfigSetting::ALL {
        app.config_setting = setting;
        let heading = match setting.category() {
            crate::app::ConfigCategory::Storage => "STORAGE",
            crate::app::ConfigCategory::Display => "DISPLAY",
            crate::app::ConfigCategory::TaskBehavior => "TASK BEHAVIOR",
            crate::app::ConfigCategory::Notifications => "NOTIFICATIONS",
        };
        for height in [3, 2, 1, 0] {
            let area = Rect::new(4, 3, 35, height);
            terminal
                .draw(|frame| super::form::draw_form(frame, &app, area))
                .unwrap();
            if height == 0 {
                continue;
            }
            let (x, y) = text_position(&terminal, "› ");
            assert!(area.contains((x, y).into()));
            if height > 1 {
                let (heading_x, heading_y) = text_position(&terminal, heading);
                assert!(area.contains((heading_x, heading_y).into()));
                assert!(heading_y < y);
            }
            let value_x = x
                + 2
                + if setting == ConfigSetting::DatabasePath {
                    0
                } else {
                    setting.name().len() as u16 + 2
                };
            assert!(
                terminal.backend().buffer()[(value_x, y)]
                    .modifier
                    .contains(Modifier::UNDERLINED)
            );
        }
    }
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
