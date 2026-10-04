use super::*;
use crate::{
    config::TaskView,
    db::{Database, Priority},
    vim_motion::{Motion, VimAction},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn app() -> App {
    let database = Database::memory();
    let root = database.add("Root", None, Priority::Mid).unwrap();
    let child = database.add("Child", Some(root), Priority::Mid).unwrap();
    database
        .add("Grandchild", Some(child), Priority::Mid)
        .unwrap();
    database.add("Sibling", Some(root), Priority::Low).unwrap();
    let other = database.add("Other root", None, Priority::Mid).unwrap();
    database
        .add("Other child", Some(other), Priority::Mid)
        .unwrap();
    App::new(database).unwrap()
}

fn press(app: &mut App, key: KeyCode) {
    app.handle_key(KeyEvent::new(key, KeyModifiers::NONE));
}

fn selected(app: &App) -> &str {
    &app.selected_todo().unwrap().title
}

fn visible(app: &App) -> Vec<&str> {
    app.visible_indices()
        .iter()
        .map(|&index| app.todos[index].title.as_str())
        .collect()
}

#[test]
fn vertical_motions_visit_visible_descendants_and_skip_collapsed_branches() {
    let mut app = app();
    press(&mut app, KeyCode::Char('j'));
    assert_eq!(selected(&app), "Child");
    press(&mut app, KeyCode::Down);
    assert_eq!(selected(&app), "Grandchild");
    press(&mut app, KeyCode::Char('k'));
    assert_eq!(selected(&app), "Child");
    press(&mut app, KeyCode::Up);
    assert_eq!(selected(&app), "Root");
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Char('j'));
    assert_eq!(selected(&app), "Other root");
    press(&mut app, KeyCode::Char('k'));
    assert_eq!(selected(&app), "Root");
    assert_eq!(visible(&app), ["Root", "Other root", "Other child"]);
    app.apply(VimAction::Move(Motion::Down, 9999)).unwrap();
    assert_eq!(selected(&app), "Other child");
    press(&mut app, KeyCode::Down);
    assert_eq!(selected(&app), "Other child");
    app.apply(VimAction::Move(Motion::Up, 9999)).unwrap();
    assert_eq!(selected(&app), "Root");
    press(&mut app, KeyCode::Up);
    assert_eq!(selected(&app), "Root");
}

#[test]
fn return_hides_descendants_keeps_selection_and_preserves_nested_folds() {
    let mut app = app();
    let saved = app.todos.clone();
    press(&mut app, KeyCode::Char('l'));
    press(&mut app, KeyCode::Enter);
    assert_eq!(selected(&app), "Child");
    assert_eq!(
        visible(&app),
        ["Root", "Child", "Sibling", "Other root", "Other child"]
    );
    press(&mut app, KeyCode::Char('h'));
    press(&mut app, KeyCode::Enter);
    assert_eq!(visible(&app), ["Root", "Other root", "Other child"]);
    assert_eq!(selected(&app), "Root");
    press(&mut app, KeyCode::Enter);
    assert_eq!(
        visible(&app),
        ["Root", "Child", "Sibling", "Other root", "Other child"]
    );
    press(&mut app, KeyCode::Char('l'));
    press(&mut app, KeyCode::Char('l'));
    assert_eq!(selected(&app), "Grandchild");
    assert_eq!(visible(&app).len(), 6);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.todos, saved);
    assert_eq!(app.database.list().unwrap(), saved);
    assert_eq!(selected(&app), "Grandchild");
}

#[test]
fn adding_and_selecting_children_reveal_collapsed_ancestors() {
    let mut app = app();
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Char('i'));
    assert!(!app.is_collapsed(app.todos[0].id));
    app.editor.insert("New child 界é👩‍💻");
    press(&mut app, KeyCode::Enter);
    assert_eq!(selected(&app), "New child 界é👩‍💻");
    press(&mut app, KeyCode::Char('h'));
    press(&mut app, KeyCode::Enter);
    let grandchild = app
        .todos
        .iter()
        .find(|todo| todo.title == "Grandchild")
        .unwrap()
        .id;
    app.select_id(grandchild);
    assert_eq!(selected(&app), "Grandchild");
}

#[test]
fn folded_tree_operations_include_hidden_descendants_and_search_reveals_matches() {
    let mut app = app();
    let saved = app.todos.clone();
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Char('x'));
    assert!(
        app.todos
            .iter()
            .filter(|todo| !todo.title.starts_with("Other"))
            .all(|todo| todo.done)
    );
    assert_eq!(visible(&app).len(), 3);
    press(&mut app, KeyCode::Char(' '));
    assert_eq!(app.todos, saved);
    app.apply(VimAction::Delete(1)).unwrap();
    assert_eq!(app.todos.len(), 2);
    app.apply(VimAction::Undo).unwrap();
    assert_eq!(app.todos, saved);
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Char('/'));
    app.editor.insert("Grandchild");
    press(&mut app, KeyCode::Enter);
    assert_eq!(visible(&app), ["Root", "Child", "Grandchild"]);
}

#[test]
fn folding_preserves_unrelated_pane_selection_and_skips_ghosts() {
    let mut app = app();
    app.config = Some(crate::config::Config {
        path: "/unused/config.toml".into(),
        database_path: "db.sql".into(),
        active_database: "/unused/db.sql".into(),
        database_override: false,
        task_view: TaskView::Split,
        default_priority: Priority::Mid,
        show_completed: true,
        sort_order: crate::config::SortOrder::Priority,
        show_hints: true,
    });
    assert_eq!(app.task_view(), TaskView::Split);
    let child = app
        .todos
        .iter()
        .find(|todo| todo.title == "Child")
        .unwrap()
        .id;
    let other_child = app
        .todos
        .iter()
        .find(|todo| todo.title == "Other child")
        .unwrap()
        .id;
    app.database.toggle(child).unwrap();
    app.database.toggle(other_child).unwrap();
    app.reload(None).unwrap();
    press(&mut app, KeyCode::Tab);
    app.apply(VimAction::FileEnd).unwrap();
    assert_eq!(selected(&app), "Other child");
    press(&mut app, KeyCode::Tab);
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Tab);
    assert_eq!(selected(&app), "Other child");
    app.apply(VimAction::FileStart).unwrap();
    assert_eq!(selected(&app), "Other child");
    press(&mut app, KeyCode::Tab);
    let root = app.selected_todo().unwrap().id;
    app.expand_task(root);
    assert_eq!(selected(&app), "Root");
    press(&mut app, KeyCode::Tab);
    assert_eq!(selected(&app), "Other child");
    press(&mut app, KeyCode::Char('h'));
    assert_eq!(app.task_pane(), TaskPane::Todo);
    assert_eq!(selected(&app), "Other root");
}

#[test]
fn return_on_empty_list_is_harmless() {
    let mut app = App::new(Database::memory()).unwrap();
    press(&mut app, KeyCode::Enter);
    assert!(app.selected_todo().is_none());
    assert!(app.collapsed.is_empty());
    assert!(!app.error);
}
