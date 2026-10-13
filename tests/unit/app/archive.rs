use crate::{
    app::App,
    config::{Config, SortOrder, TaskView},
    db::{Database, DeleteMode, Priority, TaskScope},
    vim_motion::{Motion, VimAction},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn press(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
}

#[test]
fn archive_shows_every_project_despite_search_and_preserves_live_state() {
    for view in [TaskView::Normal, TaskView::Split, TaskView::Projects] {
        let mut db = Database::memory();
        let parent = db.add("Live parent", None, Priority::Mid).unwrap();
        let child = db
            .add("Archived child", Some(parent), Priority::Mid)
            .unwrap();
        let project = db.create_project("Work").unwrap();
        let work = db
            .add_project_root("Archived work", Priority::Low, Some(project))
            .unwrap();
        for id in [child, work] {
            db.update_note(id, "Saved note 界é👩‍💻").unwrap();
            db.toggle(id).unwrap();
        }
        db.delete_completed(TaskScope::All, DeleteMode::Soft)
            .unwrap();
        let mut app = App::new(db).unwrap();
        app.config = Some(Config {
            path: "/unused/config.toml".into(),
            database_path: "db.sql".into(),
            active_database: "/unused/db.sql".into(),
            database_override: false,
            task_view: view,
            default_priority: Priority::Mid,
            show_completed: false,
            sort_order: SortOrder::Priority,
            show_hints: true,
            completed_delete: DeleteMode::Soft,
            notifications: true,
            notify_reminder_1_minutes: 60,
            notify_reminder_2_minutes: 1440,
        });
        app.query = "Live parent".into();
        app.normalize_selection();
        app.collapsed.insert(parent);
        let selection = *app.active_list();
        let pane = app.task_pane();
        let todos = app.todos.clone();
        press(&mut app, KeyCode::Char('A'));
        let archive = app.archive.as_ref().unwrap();
        assert_eq!(archive.todos.len(), 2);
        assert!(archive.todos.iter().all(|todo| todo.deleted && todo.done));
        assert_eq!(archive.selected().unwrap().parent_id, Some(parent));
        assert_eq!(archive.project_name(Some(project)), "Work");
        assert_eq!(app.note_todo().unwrap().note, "Saved note 界é👩‍💻");
        press(&mut app, KeyCode::Char('j'));
        assert_eq!(app.note_todo().unwrap().id, work);
        for key in ['x', 'd', 'd', 'n', 'e', 'i', 'P', 'C', 'u', 'm', '/'] {
            press(&mut app, KeyCode::Char(key));
        }
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));
        app.handle_paste("Do not edit");
        assert!(app.vim.input().is_none());
        assert!(app.undo.is_empty());
        assert_eq!(app.database.list().unwrap(), todos);
        assert_eq!(app.database.list_deleted().unwrap().len(), 2);
        press(&mut app, KeyCode::Char('?'));
        assert!(app.help && app.archive.is_some());
        press(&mut app, KeyCode::Esc);
        assert!(!app.help && app.archive.is_some());
        press(&mut app, KeyCode::Esc);
        assert!(app.archive.is_none() && !app.configuring);
        assert_eq!(*app.active_list(), selection);
        assert_eq!(app.task_pane(), pane);
        assert_eq!(app.query, "Live parent");
        assert!(app.collapsed.contains(&parent));
        assert_eq!(app.todos, todos);
        assert_eq!(app.active_project_id(), None);
    }
}

#[test]
fn archive_navigation_and_refresh_keep_selection_and_handle_empty_results() {
    let mut db = Database::memory();
    let ids = (0..3)
        .map(|index| {
            let id = db
                .add(&format!("Archived {index}"), None, Priority::Mid)
                .unwrap();
            db.toggle(id).unwrap();
            id
        })
        .collect::<Vec<_>>();
    let deletion = db
        .delete_completed(TaskScope::All, DeleteMode::Soft)
        .unwrap();
    let mut app = App::new(db).unwrap();
    press(&mut app, KeyCode::Char('A'));
    press(&mut app, KeyCode::Char('G'));
    assert_eq!(app.note_todo().unwrap().id, ids[2]);
    app.apply(VimAction::Refresh).unwrap();
    assert_eq!(app.note_todo().unwrap().id, ids[2]);
    press(&mut app, KeyCode::Char('g'));
    press(&mut app, KeyCode::Char('g'));
    assert_eq!(app.note_todo().unwrap().id, ids[0]);
    app.database.restore_completed(&deletion).unwrap();
    app.apply(VimAction::Refresh).unwrap();
    assert!(app.archive.as_ref().unwrap().todos.is_empty());
    for action in [
        VimAction::FileStart,
        VimAction::FileEnd,
        VimAction::Move(Motion::Down, 9999),
        VimAction::Move(Motion::Up, 9999),
    ] {
        app.apply(action).unwrap();
        assert!(app.archive.as_ref().unwrap().list.selected().is_none());
    }
    press(&mut app, KeyCode::Char('A'));
    assert!(app.archive.is_none());
    // A live refresh after leaving the archive picks up external restoration.
    app.apply(VimAction::Refresh).unwrap();
    assert_eq!(app.todos.len(), 3);
}

#[test]
fn archive_load_errors_leave_the_current_view_usable() {
    let folder = std::env::temp_dir().join(format!("argv-archive-errors-{}", std::process::id()));
    let path = folder.join("db.sql");
    let db = Database::open(&path).unwrap();
    db.add("Live", None, Priority::Mid).unwrap();
    let mut app = App::new(db).unwrap();
    let external = rusqlite::Connection::open(&path).unwrap();
    external
        .execute_batch("ALTER TABLE todos RENAME TO unavailable_todos;")
        .unwrap();
    press(&mut app, KeyCode::Char('A'));
    assert!(app.archive.is_none() && app.error);
    assert!(app.status.contains("Could not load soft-deleted tasks"));
    assert_eq!(app.selected_todo().unwrap().title, "Live");
    external
        .execute_batch("ALTER TABLE unavailable_todos RENAME TO todos;")
        .unwrap();
    press(&mut app, KeyCode::Char('A'));
    assert!(app.archive.is_some() && !app.error);
    external
        .execute_batch("ALTER TABLE projects RENAME TO unavailable_projects;")
        .unwrap();
    app.dispatch(VimAction::Refresh);
    assert!(app.error && app.archive.is_some());
    press(&mut app, KeyCode::Esc);
    assert!(app.archive.is_none() && !app.error);
    drop(external);
    drop(app);
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn separate_archive_keys_restore_and_clear_without_changing_live_focus() {
    let mut db = Database::memory();
    let live = db.add("Live", None, Priority::Mid).unwrap();
    let root = db.add("Archived", None, Priority::High).unwrap();
    let child = db.add("Child", Some(root), Priority::Low).unwrap();
    db.update_note(child, "Keep note").unwrap();
    db.toggle(root).unwrap();
    db.delete_completed(TaskScope::All, DeleteMode::Soft)
        .unwrap();
    let mut app = App::new(db).unwrap();
    app.query = "Live".into();
    app.collapsed.insert(live);
    press(&mut app, KeyCode::Char('A'));
    press(&mut app, KeyCode::Char('r'));
    assert!(!app.error);
    assert!(app.archive.as_ref().unwrap().todos.is_empty());
    assert_eq!(app.todos.len(), 3);
    assert_eq!(app.selected_todo().unwrap().id, live);
    assert_eq!(app.query, "Live");
    assert!(app.collapsed.contains(&live));
    assert_eq!(
        app.todos.iter().find(|task| task.id == child).unwrap().note,
        "Keep note"
    );
    app.database
        .delete_completed(TaskScope::All, DeleteMode::Soft)
        .unwrap();
    press(&mut app, KeyCode::Char('D'));
    assert_eq!(app.archive_clear_count(), Some(2));
    for key in [KeyCode::Char('r'), KeyCode::Char('j'), KeyCode::Char('D')] {
        press(&mut app, key);
        assert_eq!(app.database.list_deleted().unwrap().len(), 2);
    }
    app.handle_paste("Do not confirm");
    press(&mut app, KeyCode::Esc);
    assert!(app.archive.is_some() && app.archive_clear_count().is_none());
    assert_eq!(app.database.list_deleted().unwrap().len(), 2);
    press(&mut app, KeyCode::Char('D'));
    for kind in [
        crossterm::event::KeyEventKind::Repeat,
        crossterm::event::KeyEventKind::Release,
    ] {
        app.handle_key(KeyEvent::new_with_kind(
            KeyCode::Enter,
            KeyModifiers::NONE,
            kind,
        ));
        assert_eq!(app.database.list_deleted().unwrap().len(), 2);
    }
    press(&mut app, KeyCode::Enter);
    assert!(!app.error && app.archive_clear_count().is_none());
    assert!(app.archive.as_ref().unwrap().todos.is_empty());
    assert_eq!(app.todos.len(), 1);
    assert_eq!(app.selected_todo().unwrap().id, live);
    press(&mut app, KeyCode::Char('r'));
    press(&mut app, KeyCode::Char('D'));
    assert!(!app.error && app.archive_clear_count().is_none());
    press(&mut app, KeyCode::Esc);
    assert!(app.archive.is_none() && !app.configuring);
}

#[test]
fn archive_mutations_retire_only_overlapping_undo_and_stale_clear_is_safe() {
    for restore in [false, true] {
        let db = Database::memory();
        let archived = db.add("Archived", None, Priority::Mid).unwrap();
        db.add("Other deletion", None, Priority::Low).unwrap();
        let mut app = App::new(db).unwrap();
        app.apply(VimAction::Toggle).unwrap();
        app.apply(VimAction::ClearCompleted).unwrap();
        app.apply(VimAction::Submit).unwrap();
        app.apply(VimAction::Delete(1)).unwrap();
        app.apply(VimAction::Submit).unwrap();
        assert_eq!(app.undo.len(), 2);
        press(&mut app, KeyCode::Char('A'));
        if restore {
            press(&mut app, KeyCode::Char('r'));
        } else {
            press(&mut app, KeyCode::Char('D'));
            press(&mut app, KeyCode::Enter);
        }
        assert!(!app.error);
        assert_eq!(app.undo.len(), 1);
        press(&mut app, KeyCode::Esc);
        press(&mut app, KeyCode::Char('u'));
        assert!(!app.error);
        assert!(app.todos.iter().any(|task| task.title == "Other deletion"));
        assert_eq!(app.todos.iter().any(|task| task.id == archived), restore);
    }
    let mut db = Database::memory();
    let id = db.add("Archived", None, Priority::Mid).unwrap();
    db.toggle(id).unwrap();
    db.delete_completed(TaskScope::All, DeleteMode::Soft)
        .unwrap();
    let mut app = App::new(db).unwrap();
    press(&mut app, KeyCode::Char('A'));
    press(&mut app, KeyCode::Char('D'));
    app.database.restore_archived_tree(id).unwrap();
    press(&mut app, KeyCode::Enter);
    assert!(app.error && app.archive.is_some());
    assert!(app.archive_clear_count().is_none());
    assert_eq!(app.database.list().unwrap().len(), 1);
    press(&mut app, KeyCode::Esc);
    assert!(app.archive.is_none());
}
