use super::*;
use crate::{
    config::{Config, SortOrder, TaskView},
    db::{Database, DeleteMode, Priority, TaskScope},
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
