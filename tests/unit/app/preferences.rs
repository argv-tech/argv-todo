use super::*;
use crate::{
    app::{ConfigSetting, TaskPane},
    config::Config,
    db::{Database, TaskMove},
    vim_motion::{Motion, VimAction},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn app() -> (std::path::PathBuf, App) {
    let folder = std::env::temp_dir().join(format!(
        "argv-todo-app-preferences-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut app = App::new(Database::memory()).unwrap();
    app.config = Some(Config::load(&folder.join("db.sql"), false).unwrap());
    (folder, app)
}

fn press(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
    assert!(!app.error, "{}", app.status);
}

fn change(app: &mut App, setting: ConfigSetting) {
    app.configuring = true;
    app.config_setting = setting;
    press(app, KeyCode::Enter);
    app.configuring = false;
}

#[test]
fn root_defaults_apply_immediately_children_and_siblings_keep_inheritance() {
    let (folder, mut app) = app();
    change(&mut app, ConfigSetting::DefaultPriority);
    assert_eq!(app.default_priority(), Priority::High);
    app.begin_add(None);
    assert_eq!(app.input_priority, Priority::High);
    app.editor.insert("New root");
    app.apply(VimAction::Submit).unwrap();
    let root = app.selected_todo().unwrap().id;
    app.apply(VimAction::Priority(Priority::Low)).unwrap();
    app.begin_add(Some(root));
    assert_eq!(app.input_priority, Priority::Low);
    app.editor.insert("Child");
    app.apply(VimAction::Submit).unwrap();
    app.begin_sibling(false);
    assert_eq!(app.input_priority, Priority::Low);
    app.apply(VimAction::Cancel).unwrap();
    assert_eq!(
        app.todos
            .iter()
            .find(|todo| todo.id == root)
            .unwrap()
            .priority,
        Priority::Low
    );
    assert_eq!(
        Config::load(&folder.join("db.sql"), false)
            .unwrap()
            .default_priority,
        Priority::High
    );
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn completion_visibility_preserves_selection_and_split_forces_false_without_losing_preference() {
    let (folder, mut app) = app();
    let root = app.database.add("Root", None, Priority::Mid).unwrap();
    let child = app
        .database
        .add("Child", Some(root), Priority::Mid)
        .unwrap();
    let other = app.database.add("Other", None, Priority::Mid).unwrap();
    app.database.toggle(child).unwrap();
    app.reload(Some(root)).unwrap();
    change(&mut app, ConfigSetting::ShowCompleted);
    assert!(!app.show_completed());
    assert_eq!(app.selected_todo().unwrap().id, root);
    assert_eq!(
        app.visible_indices()
            .iter()
            .map(|&index| app.todos[index].id)
            .collect::<Vec<_>>(),
        [root, other]
    );
    app.apply(VimAction::Move(Motion::Right, 1)).unwrap();
    assert_eq!(app.selected_todo().unwrap().id, root);
    app.apply(VimAction::Toggle).unwrap();
    assert_eq!(app.selected_todo().unwrap().id, other);
    change(&mut app, ConfigSetting::ShowCompleted);
    assert!(app.show_completed());
    change(&mut app, ConfigSetting::TaskView);
    assert!(!app.show_completed());
    assert!(app.config.as_ref().unwrap().show_completed);
    let before = std::fs::read_to_string(&app.config.as_ref().unwrap().path).unwrap();
    change(&mut app, ConfigSetting::ShowCompleted);
    assert!(!app.show_completed());
    assert_eq!(
        std::fs::read_to_string(&app.config.as_ref().unwrap().path).unwrap(),
        before
    );
    assert_eq!(
        app.rows_for_pane(TaskPane::Completed)
            .iter()
            .filter(|row| !row.ghost)
            .count(),
        2
    );
    change(&mut app, ConfigSetting::TaskView);
    assert!(app.show_completed());
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn manual_movement_keeps_subtrees_selection_and_hidden_descendants() {
    let (folder, mut app) = app();
    let first = app.database.add("First", None, Priority::Low).unwrap();
    let second = app.database.add("Second", None, Priority::High).unwrap();
    let child = app
        .database
        .add("Hidden child", Some(second), Priority::Mid)
        .unwrap();
    app.reload(Some(second)).unwrap();
    app.apply(VimAction::MoveTask(TaskMove::Down, 1)).unwrap();
    assert_eq!(
        app.visible_indices()
            .iter()
            .map(|&index| app.todos[index].id)
            .collect::<Vec<_>>(),
        [second, child, first]
    );
    change(&mut app, ConfigSetting::SortOrder);
    assert_eq!(app.sort_order(), SortOrder::Manual);
    assert_eq!(app.selected_todo().unwrap().id, second);
    assert_eq!(
        app.visible_indices()
            .iter()
            .map(|&index| app.todos[index].id)
            .collect::<Vec<_>>(),
        [first, second, child]
    );
    app.query = "Second".into();
    app.normalize_selection();
    press(&mut app, KeyCode::Char('L'));
    assert_eq!(app.selected_todo().unwrap().id, second);
    assert_eq!(app.selected_todo().unwrap().parent_id, Some(first));
    assert_eq!(
        app.todos
            .iter()
            .find(|todo| todo.id == child)
            .unwrap()
            .parent_id,
        Some(second)
    );
    press(&mut app, KeyCode::Char('H'));
    press(&mut app, KeyCode::Char('K'));
    app.query.clear();
    assert_eq!(
        app.visible_indices()
            .iter()
            .map(|&index| app.todos[index].id)
            .collect::<Vec<_>>(),
        [second, child, first]
    );
    press(&mut app, KeyCode::Char('l'));
    assert_eq!(app.selected_todo().unwrap().id, child);
    change(&mut app, ConfigSetting::TaskView);
    app.apply(VimAction::Toggle).unwrap();
    assert_eq!(app.task_pane(), TaskPane::Completed);
    press(&mut app, KeyCode::Char('H'));
    assert_eq!(app.selected_todo().unwrap().id, child);
    assert_eq!(app.selected_todo().unwrap().parent_id, None);
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn setting_navigation_cycles_all_values_and_failed_saves_leave_behavior_intact() {
    let (folder, mut app) = app();
    app.configuring = true;
    for setting in ConfigSetting::ALL {
        assert_eq!(app.config_setting, setting);
        press(&mut app, KeyCode::Tab);
    }
    assert_eq!(app.config_setting, ConfigSetting::DatabasePath);
    press(&mut app, KeyCode::BackTab);
    assert_eq!(app.config_setting, ConfigSetting::ShowHints);
    press(&mut app, KeyCode::Enter);
    assert!(!app.show_hints());
    press(&mut app, KeyCode::Home);
    assert_eq!(app.config_setting, ConfigSetting::DatabasePath);
    press(&mut app, KeyCode::Char('2'));
    press(&mut app, KeyCode::Char('j'));
    assert_eq!(app.config_setting, ConfigSetting::DefaultPriority);
    press(&mut app, KeyCode::Char('h'));
    assert_eq!(app.default_priority(), Priority::Low);
    let blocked = app
        .config
        .as_ref()
        .unwrap()
        .path
        .with_file_name(format!(".config.toml.{}.tmp", std::process::id()));
    std::fs::write(&blocked, "blocked").unwrap();
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(app.error);
    assert_eq!(app.default_priority(), Priority::Low);
    std::fs::remove_dir_all(folder).unwrap();
}
