use super::*;
use crate::{
    config::Config,
    db::{Database, Priority},
    vim_motion::{InputTarget, VimAction},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn app(view: TaskView) -> App {
    let database = Database::memory();
    let parent = database.add("Parent", None, Priority::Mid).unwrap();
    let child = database.add("Child", Some(parent), Priority::Mid).unwrap();
    database
        .add("Grandchild", Some(child), Priority::Mid)
        .unwrap();
    database
        .add("Sibling", Some(parent), Priority::Mid)
        .unwrap();
    database.add("Other root", None, Priority::Mid).unwrap();
    let mut app = App::new(database).unwrap();
    app.config = Some(Config {
        path: "/unused/config.toml".into(),
        database_path: "db.sql".into(),
        active_database: "/unused/db.sql".into(),
        database_override: false,
        task_view: view,
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

#[test]
fn settings_save_and_apply_layouts_live_and_preserve_selection_on_return_to_normal() {
    let folder = std::env::temp_dir().join(format!(
        "argv-todo-live-view-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut app = app(TaskView::Normal);
    app.config = Some(Config::load(&folder.join("db.sql"), true).unwrap());
    let tasks = app.todos.clone();
    for view in [TaskView::Split, TaskView::Normal] {
        press(&mut app, KeyCode::Esc);
        press(&mut app, KeyCode::Char('j'));
        press(&mut app, KeyCode::Enter);
        assert!(app.configuring && !app.error);
        assert_eq!(app.task_view(), view);
        let saved = Config::load(&folder.join("db.sql"), true).unwrap();
        assert_eq!(saved.task_view, view);
        assert_eq!(app.todos, tasks);
        press(&mut app, KeyCode::Esc);
        if view == TaskView::Split {
            press(&mut app, KeyCode::Char('j'));
        }
        assert_eq!(app.selected_todo().unwrap().title, "Child");
    }
    assert_eq!(app.task_pane(), TaskPane::Tree);
    assert_eq!(app.selected_todo().unwrap().title, "Child");

    // A failed save leaves both the running layout and the file unchanged.
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Char('j'));
    let config = app.config.as_ref().unwrap();
    let saved = std::fs::read_to_string(&config.path).unwrap();
    let temporary = config
        .path
        .with_file_name(format!(".config.toml.{}.tmp", std::process::id()));
    std::fs::write(&temporary, "blocked").unwrap();
    press(&mut app, KeyCode::Enter);
    assert!(app.error);
    assert_eq!(app.task_view(), TaskView::Normal);
    assert_eq!(
        std::fs::read_to_string(&app.config.as_ref().unwrap().path).unwrap(),
        saved
    );
    std::fs::remove_file(temporary).unwrap();
    // External settings are applied when configuration is reopened.
    press(&mut app, KeyCode::Esc);
    std::fs::write(&app.config.as_ref().unwrap().path, "task_view = 'split'").unwrap();
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.task_view(), TaskView::Split);
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn split_filters_completion_and_tab_targets_each_list() {
    let mut app = app(TaskView::Split);
    let child = app
        .todos
        .iter()
        .find(|todo| todo.title == "Child")
        .unwrap()
        .id;
    app.database.toggle(child).unwrap();
    app.reload(None).unwrap();
    assert_eq!(app.task_pane(), TaskPane::Todo);
    assert_eq!(
        app.rows_for_pane(TaskPane::Todo)
            .iter()
            .map(|row| (app.todos[row.index].title.as_str(), row.depth))
            .collect::<Vec<_>>(),
        [("Parent", 0), ("Sibling", 1), ("Other root", 0)]
    );
    assert_eq!(
        app.rows_for_pane(TaskPane::Completed)
            .iter()
            .map(|row| (app.todos[row.index].title.as_str(), row.depth, row.ghost))
            .collect::<Vec<_>>(),
        [
            ("Parent", 0, true),
            ("Child", 1, false),
            ("Grandchild", 2, false)
        ]
    );
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.task_pane(), TaskPane::Completed);
    assert_eq!(app.selected_todo().unwrap().title, "Child");
    press(&mut app, KeyCode::Char('j'));
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.selected_todo().unwrap().title, "Parent");
    press(&mut app, KeyCode::BackTab);
    assert_eq!(app.selected_todo().unwrap().title, "Grandchild");
    press(&mut app, KeyCode::Char('x'));
    assert_eq!(app.task_pane(), TaskPane::Todo);
    assert_eq!(app.selected_todo().unwrap().title, "Grandchild");
    assert!(!app.selected_todo().unwrap().done);
    assert_eq!(
        app.rows_for_pane(TaskPane::Completed)
            .iter()
            .filter(|row| !row.ghost)
            .count(),
        1
    );
    app.apply(VimAction::Delete(1)).unwrap();
    app.apply(VimAction::Undo).unwrap();
    assert_eq!(app.selected_todo().unwrap().title, "Grandchild");
    press(&mut app, KeyCode::Char('h'));
    assert_eq!(app.task_pane(), TaskPane::Completed);
    assert_eq!(app.selected_todo().unwrap().title, "Child");
    press(&mut app, KeyCode::Char('h'));
    assert_eq!(app.task_pane(), TaskPane::Todo);
    assert_eq!(app.selected_todo().unwrap().title, "Parent");
}

#[test]
fn split_search_and_tree_operations_include_descendants_in_both_panes() {
    let mut app = app(TaskView::Split);
    let child = app
        .todos
        .iter()
        .find(|todo| todo.title == "Child")
        .unwrap()
        .id;
    app.database.toggle(child).unwrap();
    app.reload(None).unwrap();
    press(&mut app, KeyCode::Char('/'));
    app.editor.insert("Child");
    app.normalize_selection();
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.rows_for_pane(TaskPane::Todo).len(), 1);
    assert_eq!(app.rows_for_pane(TaskPane::Completed).len(), 3);
    app.apply(VimAction::Delete(1)).unwrap();
    assert_eq!(app.todos.len(), 1);
    assert_eq!(app.todos[0].title, "Other root");
    app.apply(VimAction::Undo).unwrap();
    assert_eq!(app.todos.len(), 5);
    assert_eq!(app.rows_for_pane(TaskPane::Completed).len(), 3);
    press(&mut app, KeyCode::Char('x'));
    assert_eq!(app.task_pane(), TaskPane::Completed);
    assert_eq!(app.selected_todo().unwrap().title, "Parent");
    assert_eq!(app.rows_for_pane(TaskPane::Completed).len(), 4);
    press(&mut app, KeyCode::Char('x'));
    assert_eq!(app.rows_for_pane(TaskPane::Completed).len(), 0);
    assert_eq!(app.task_pane(), TaskPane::Todo);
}

#[test]
fn completed_grandchildren_keep_ghost_ancestors_and_navigation_skips_them() {
    let mut app = app(TaskView::Split);
    for title in ["Grandchild", "Sibling"] {
        let id = app
            .todos
            .iter()
            .find(|todo| todo.title == title)
            .unwrap()
            .id;
        app.database.toggle(id).unwrap();
    }
    app.reload(None).unwrap();
    let rows = app.rows_for_pane(TaskPane::Completed);
    assert_eq!(
        rows.iter()
            .map(|row| (app.todos[row.index].title.as_str(), row.depth, row.ghost))
            .collect::<Vec<_>>(),
        [
            ("Parent", 0, true),
            ("Child", 1, true),
            ("Grandchild", 2, false),
            ("Sibling", 1, false)
        ]
    );
    press(&mut app, KeyCode::Tab);
    app.apply(VimAction::FileStart).unwrap();
    assert_eq!(app.selected_todo().unwrap().title, "Grandchild");
    press(&mut app, KeyCode::Char('k'));
    assert_eq!(app.selected_todo().unwrap().title, "Grandchild");
    press(&mut app, KeyCode::Char('j'));
    assert_eq!(app.selected_todo().unwrap().title, "Sibling");
    app.apply(VimAction::FileStart).unwrap();
    app.apply(VimAction::Delete(2)).unwrap();
    assert_eq!(
        app.todos
            .iter()
            .map(|todo| todo.title.as_str())
            .collect::<Vec<_>>(),
        ["Parent", "Child", "Other root"]
    );
    app.apply(VimAction::Undo).unwrap();
    assert_eq!(app.selected_todo().unwrap().title, "Grandchild");
    assert!(
        app.todos
            .iter()
            .filter(|todo| matches!(todo.title.as_str(), "Parent" | "Child"))
            .all(|todo| !todo.done)
    );
}

#[test]
fn normal_tab_keeps_selection_and_split_fields_keep_focus() {
    let mut normal = app(TaskView::Normal);
    let selected = normal.selected_todo().unwrap().id;
    press(&mut normal, KeyCode::Tab);
    assert_eq!(normal.task_pane(), TaskPane::Tree);
    assert_eq!(normal.selected_todo().unwrap().id, selected);
    let mut split = app(TaskView::Split);
    press(&mut split, KeyCode::Tab);
    assert_eq!(split.task_pane(), TaskPane::Completed);
    assert!(split.selected_todo().is_none());
    press(&mut split, KeyCode::Char('i'));
    assert_eq!(split.adding_parent, None);
    split.editor.insert("界é👩‍💻");
    press(&mut split, KeyCode::Tab);
    assert_eq!(split.task_pane(), TaskPane::Completed);
    assert_eq!(split.vim.input(), Some(InputTarget::Task));
    press(&mut split, KeyCode::Enter);
    assert_eq!(split.task_pane(), TaskPane::Todo);
    assert_eq!(split.selected_todo().unwrap().title, "界é👩‍💻");
    assert_eq!(split.selected_todo().unwrap().parent_id, None);
}
