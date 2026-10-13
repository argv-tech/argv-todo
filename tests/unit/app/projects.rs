use crate::{
    app::App,
    config::{Config, TaskView},
    db::{Database, Priority},
    vim_motion::{InputTarget, VimAction},
};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::path::PathBuf;

fn app() -> (PathBuf, App) {
    let folder = std::env::temp_dir().join(format!(
        "argv-projects-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let database = Database::memory();
    let inbox = database.add("Inbox task", None, Priority::Mid).unwrap();
    database.update_note(inbox, "Inbox note").unwrap();
    let work = database.create_project("Work").unwrap();
    let root = database
        .add_project_root("Work root", Priority::Mid, Some(work))
        .unwrap();
    database
        .add("Work child", Some(root), Priority::Mid)
        .unwrap();
    database.create_project("Empty").unwrap();
    let mut app = App::new(database).unwrap();
    let mut config = Config::load(&folder.join("db.sql"), true).unwrap();
    config.save_task_view(TaskView::Projects).unwrap();
    app.config = Some(config);
    app.normalize_selection();
    (folder, app)
}

fn press(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
}

#[test]
fn project_tabs_scope_navigation_search_and_keep_independent_selection() {
    let (folder, mut app) = app();
    assert_eq!(
        app.project_tabs().map(|(_, name)| name).collect::<Vec<_>>(),
        ["Inbox", "Work", "Empty"]
    );
    assert_eq!(app.selected_todo().unwrap().title, "Inbox task");
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.selected_todo().unwrap().title, "Work root");
    press(&mut app, KeyCode::Char('j'));
    *app.project_list_mut().offset_mut() = 1;
    press(&mut app, KeyCode::Tab);
    assert!(app.selected_todo().is_none());
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.selected_todo().unwrap().note, "Inbox note");
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.selected_todo().unwrap().title, "Work child");
    assert_eq!(app.project_list().offset(), 1);
    app.query = "Inbox".into();
    app.normalize_selection();
    assert!(app.visible_rows().is_empty());
    app.query.clear();
    press(&mut app, KeyCode::BackTab);
    assert_eq!(app.selected_todo().unwrap().title, "Inbox task");
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn project_creation_validation_paste_and_task_inheritance() {
    let (folder, mut app) = app();
    press(&mut app, KeyCode::Char('P'));
    assert_eq!(app.vim.input(), Some(InputTarget::ProjectName));
    press(&mut app, KeyCode::Enter);
    assert!(app.error && app.vim.input().is_some());
    app.handle_paste("New 界é👩‍💻");
    press(&mut app, KeyCode::Enter);
    assert!(!app.error);
    assert_eq!(
        app.project_tabs()
            .nth(app.active_project_index())
            .unwrap()
            .1,
        "New 界é👩‍💻"
    );
    assert!(app.selected_todo().is_none());
    let project = app.active_project_id();
    press(&mut app, KeyCode::Char('i'));
    app.handle_paste("New root");
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.selected_todo().unwrap().project_id, project);
    press(&mut app, KeyCode::Char('i'));
    app.handle_paste("New child");
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.selected_todo().unwrap().project_id, project);
    assert!(app.selected_todo().unwrap().parent_id.is_some());
    press(&mut app, KeyCode::Char('P'));
    app.handle_paste("new 界é👩‍💻");
    press(&mut app, KeyCode::Enter);
    assert!(app.error);
    assert_eq!(app.project_tabs().len(), 4);
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Esc);
    assert!(app.vim.input().is_none());
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn move_picker_isolates_input_moves_whole_tree_and_undo_preserves_project() {
    let (folder, mut app) = app();
    press(&mut app, KeyCode::Tab);
    press(&mut app, KeyCode::Char('l'));
    let id = app.selected_todo().unwrap().id;
    press(&mut app, KeyCode::Char('m'));
    let before = app.todos.clone();
    for code in [
        KeyCode::Char('x'),
        KeyCode::Char('d'),
        KeyCode::Char('d'),
        KeyCode::Char('P'),
        KeyCode::Char('i'),
    ] {
        press(&mut app, code);
    }
    app.handle_paste("Ignored paste");
    assert_eq!(app.todos, before);
    press(&mut app, KeyCode::Home);
    app.handle_key(KeyEvent::new_with_kind(
        KeyCode::Enter,
        KeyModifiers::NONE,
        KeyEventKind::Repeat,
    ));
    assert!(app.project_picker().is_some());
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.todos, before);
    press(&mut app, KeyCode::Char('m'));
    press(&mut app, KeyCode::Home);
    press(&mut app, KeyCode::Enter);
    assert!(app.project_picker().is_none());
    assert_eq!(app.active_project_id(), None);
    assert_eq!(app.selected_todo().unwrap().id, id);
    assert_eq!(app.selected_todo().unwrap().parent_id, None);
    app.apply(VimAction::Delete(1)).unwrap();
    app.apply(VimAction::Undo).unwrap();
    assert_eq!(app.selected_todo().unwrap().id, id);
    assert_eq!(app.selected_todo().unwrap().project_id, None);
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn new_project_from_normal_saves_layout_and_picker_errors_remain_cancellable() {
    let (folder, mut app) = app();
    app.config
        .as_mut()
        .unwrap()
        .save_task_view(TaskView::Normal)
        .unwrap();
    press(&mut app, KeyCode::Char('P'));
    assert_eq!(app.task_view(), TaskView::Projects);
    assert_eq!(
        Config::load(&folder.join("db.sql"), true)
            .unwrap()
            .task_view,
        TaskView::Projects
    );
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Esc);
    let task = app.selected_todo().unwrap().clone();
    press(&mut app, KeyCode::Char('m'));
    app.database.delete(&[task]).unwrap();
    press(&mut app, KeyCode::Enter);
    assert!(app.error && app.project_picker().is_some());
    press(&mut app, KeyCode::Esc);
    assert!(app.project_picker().is_none());
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn project_layout_cycle_honors_reverse_counts_and_failed_saves() {
    let (folder, mut app) = app();
    app.config
        .as_mut()
        .unwrap()
        .save_task_view(TaskView::Normal)
        .unwrap();
    app.configuring = true;
    app.config_setting = crate::app::ConfigSetting::TaskView;
    press(&mut app, KeyCode::Char('h'));
    assert_eq!(app.task_view(), TaskView::Projects);
    press(&mut app, KeyCode::Char('2'));
    press(&mut app, KeyCode::Char('l'));
    assert_eq!(app.task_view(), TaskView::Split);
    press(&mut app, KeyCode::Char('3'));
    press(&mut app, KeyCode::Char('h'));
    assert_eq!(app.task_view(), TaskView::Split);
    app.config
        .as_mut()
        .unwrap()
        .save_task_view(TaskView::Normal)
        .unwrap();
    app.configuring = false;
    let saved = std::fs::read_to_string(&app.config.as_ref().unwrap().path).unwrap();
    let blocked = app
        .config
        .as_ref()
        .unwrap()
        .path
        .with_file_name(format!(".config.toml.{}.tmp", std::process::id()));
    std::fs::write(blocked, "blocked").unwrap();
    let projects = app.database.projects().unwrap();
    press(&mut app, KeyCode::Char('P'));
    assert!(app.error);
    assert!(app.vim.input().is_none());
    assert_eq!(app.task_view(), TaskView::Normal);
    assert_eq!(app.database.projects().unwrap(), projects);
    assert_eq!(
        std::fs::read_to_string(&app.config.as_ref().unwrap().path).unwrap(),
        saved
    );
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn conversion_selects_new_project_and_keeps_notes_and_hidden_descendants() {
    let (folder, mut app) = app();
    app.config
        .as_mut()
        .unwrap()
        .save_task_view(TaskView::Normal)
        .unwrap();
    let root = app
        .todos
        .iter()
        .find(|task| task.title == "Work root")
        .unwrap()
        .id;
    app.select_id(root);
    app.toggle_collapse();
    app.query = "Work root".into();
    app.normalize_selection();
    app.handle_key(KeyEvent::new(KeyCode::Char('C'), KeyModifiers::SHIFT));
    assert!(!app.error);
    assert_eq!(app.task_view(), TaskView::Projects);
    assert_eq!(
        app.project_tabs()
            .nth(app.active_project_index())
            .unwrap()
            .1,
        "Work root"
    );
    assert!(app.query.is_empty());
    assert_eq!(app.selected_todo().unwrap().id, root);
    let project = app.active_project_id();
    assert!(
        app.todos
            .iter()
            .filter(|task| task.title.starts_with("Work "))
            .all(|task| task.project_id == project)
    );
    assert!(app.is_collapsed(root));
    press(&mut app, KeyCode::Char('l'));
    let before = app.todos.clone();
    press(&mut app, KeyCode::Char('C'));
    assert!(app.error);
    assert_eq!(app.todos, before);
    assert_eq!(app.project_tabs().len(), 4);
    press(&mut app, KeyCode::Char('h'));
    press(&mut app, KeyCode::Char('C'));
    assert!(app.error);
    assert_eq!(app.todos, before);
    std::fs::remove_dir_all(folder).unwrap();
}
