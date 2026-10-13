use crate::{
    app::App,
    config::{Config, TaskView},
    db::{Database, DeleteMode, Priority},
    vim_motion::{InputTarget, VimAction},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[test]
fn completed_cleanup_ignores_search_and_folds_and_supports_both_undo_modes() {
    for mode in [DeleteMode::Soft, DeleteMode::Hard] {
        let folder = std::env::temp_dir().join(format!(
            "argv-cleanup-{}-{}",
            std::process::id(),
            mode.label()
        ));
        let mut db = Database::memory();
        let root = db.add("Root", None, Priority::Mid).unwrap();
        let completed = db
            .add("Hidden completed", Some(root), Priority::Mid)
            .unwrap();
        db.toggle(completed).unwrap();
        let mut app = App::new(db).unwrap();
        app.config = Some(Config::load(&folder.join("db.sql"), true).unwrap());
        app.config
            .as_mut()
            .unwrap()
            .save_completed_delete(mode)
            .unwrap();
        let before = app.todos.clone();
        app.query = "Root".into();
        app.toggle_collapse();
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));
        assert!(!app.error);
        assert_eq!(app.todos.len(), 1);
        assert_eq!(app.selected_todo().unwrap().id, root);
        assert_eq!(app.undo.len(), 1);
        app.apply(VimAction::Delete(1)).unwrap();
        assert!(app.todos.is_empty());
        app.apply(VimAction::Undo).unwrap();
        assert_eq!(app.todos.len(), 1);
        assert_eq!(app.selected_todo().unwrap().id, root);
        app.apply(VimAction::Undo).unwrap();
        assert_eq!(app.todos, before);
        assert_eq!(app.selected_todo().unwrap().id, completed);
        app.apply(VimAction::ClearCompleted).unwrap();
        let undo_count = app.undo.len();
        app.apply(VimAction::ClearCompleted).unwrap();
        assert_eq!(app.undo.len(), undo_count);
        std::fs::remove_dir_all(folder).unwrap();
    }
}

#[test]
fn completed_deletion_setting_changes_live_and_persists() {
    let folder = std::env::temp_dir().join(format!("argv-cleanup-setting-{}", std::process::id()));
    let mut app = App::new(Database::memory()).unwrap();
    app.config = Some(Config::load(&folder.join("db.sql"), true).unwrap());
    app.configuring = true;
    app.config_setting = crate::app::ConfigSetting::CompletedDelete;
    assert_eq!(app.completed_delete(), DeleteMode::Soft);
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(!app.error);
    assert_eq!(app.completed_delete(), DeleteMode::Hard);
    assert_eq!(
        Config::load(&folder.join("db.sql"), true)
            .unwrap()
            .completed_delete,
        DeleteMode::Hard
    );
    app.handle_key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE));
    assert_eq!(app.completed_delete(), DeleteMode::Soft);
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn completed_cleanup_is_scoped_to_project_and_fields_and_picker_block_shortcut() {
    let mut db = Database::memory();
    let inbox = db.add("Inbox", None, Priority::Mid).unwrap();
    db.toggle(inbox).unwrap();
    let project = db.create_project("Work").unwrap();
    let work = db
        .add_project_root("Work task", Priority::Mid, Some(project))
        .unwrap();
    db.toggle(work).unwrap();
    let folder = std::env::temp_dir().join(format!("argv-cleanup-scope-{}", std::process::id()));
    let mut app = App::new(db).unwrap();
    let mut config = Config::load(&folder.join("db.sql"), true).unwrap();
    config.save_task_view(TaskView::Projects).unwrap();
    app.config = Some(config);
    app.normalize_selection();
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));
    assert_eq!(app.todos.len(), 1);
    assert_eq!(app.todos[0].id, work);
    assert!(app.selected_todo().is_none());
    app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    let before = app.todos.clone();
    app.vim.begin_input(InputTarget::Note);
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));
    assert_eq!(app.todos, before);
    app.vim.end_input();
    app.begin_project_move();
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));
    assert_eq!(app.todos, before);
    assert!(app.project_picker().is_some());
    app.apply(VimAction::Cancel).unwrap();
    app.apply(VimAction::Undo).unwrap();
    assert_eq!(app.active_project_id(), None);
    assert_eq!(app.selected_todo().unwrap().id, inbox);
    std::fs::remove_dir_all(folder).unwrap();
}
