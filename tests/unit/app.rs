use super::*;
use crate::vim_motion::{VimAction, VimMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;
fn app() -> App {
    App::new(Database::memory()).unwrap()
}
fn keys(app: &mut App, text: &str) {
    for c in text.chars() {
        if let Some(action) = app
            .vim
            .handle(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
        {
            app.dispatch(action);
        }
    }
}
fn enter(app: &mut App) {
    let action = app
        .vim
        .handle(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .unwrap();
    app.dispatch(action);
}

fn cycle_input_priority(app: &mut App) {
    let action = app
        .vim
        .handle(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL))
        .unwrap();
    app.dispatch(action);
}

fn escape(app: &mut App) {
    let action = app
        .vim
        .handle(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
        .unwrap();
    app.dispatch(action);
}

#[test]
fn escape_opens_config_and_edits_settings_without_changing_tasks() {
    let folder = std::env::temp_dir().join(format!(
        "argv-todo-config-ui-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let config = Config::load(&folder.join("custom.sql"), true).unwrap();
    let path = config.path.clone();
    let mut app = app();
    app.config = Some(config);
    keys(&mut app, "iKeep this task");
    enter(&mut app);
    let tasks = app.todos.clone();
    let selection = app.list.selected();

    // Input exits Insert before cancellation; help and applied search close directly.
    keys(&mut app, "eChanged task");
    escape(&mut app);
    assert_eq!(app.vim.mode(), VimMode::Normal);
    assert!(app.vim.input().is_some());
    escape(&mut app);
    assert!(!app.configuring);
    assert_eq!(app.todos, tasks);
    keys(&mut app, "?");
    escape(&mut app);
    assert!(!app.help && !app.configuring);
    keys(&mut app, "/Keep");
    enter(&mut app);
    escape(&mut app);
    assert!(app.query.is_empty() && !app.configuring);

    escape(&mut app);
    assert!(app.configuring);
    keys(&mut app, "ddtu");
    assert_eq!(app.todos, tasks);
    enter(&mut app);
    assert_eq!(app.vim.mode(), VimMode::Insert);
    app.dispatch(VimAction::Clear);
    enter(&mut app);
    assert!(app.error && app.configuring);
    assert_eq!(app.vim.mode(), VimMode::Insert);
    assert_eq!(parse_config(&path), PathBuf::from("db.sql"));
    keys(&mut app, "storage/tasks.sql");
    enter(&mut app);
    assert!(!app.error && app.configuring);
    assert_eq!(app.vim.mode(), VimMode::Normal);
    assert_eq!(parse_config(&path), PathBuf::from("storage/tasks.sql"));
    assert_eq!(
        app.config.as_ref().unwrap().active_database,
        folder.join("custom.sql")
    );
    assert!(!folder.join("storage/tasks.sql").exists());

    keys(&mut app, "eDiscard this change");
    escape(&mut app);
    assert!(app.configuring);
    assert_eq!(parse_config(&path), PathBuf::from("storage/tasks.sql"));
    escape(&mut app);
    assert!(app.configuring && app.vim.input().is_none());
    escape(&mut app);
    assert!(!app.configuring);
    assert_eq!(app.todos, tasks);
    assert_eq!(app.list.selected(), selection);

    std::fs::write(&path, "database_path = 'external.sql'").unwrap();
    escape(&mut app);
    assert_eq!(app.config.as_ref().unwrap().database_path, "external.sql");
    keys(&mut app, "q");
    assert!(!app.running);
    std::fs::remove_dir_all(folder).unwrap();

    fn parse_config(path: &std::path::Path) -> std::path::PathBuf {
        let content = std::fs::read_to_string(path).unwrap();
        let value = content.parse::<toml::Table>().unwrap();
        value["database_path"].as_str().unwrap().into()
    }
}

#[test]
fn new_children_inherit_parent_priority_for_all_creation_keys() {
    for priority in [Priority::High, Priority::Mid, Priority::Low] {
        for key in ["a", "i"] {
            let database = Database::memory();
            let parent = database.add("Parent", None, priority).unwrap();
            database
                .add("Existing child", Some(parent), Priority::Low)
                .unwrap();
            let mut app = App::new(database).unwrap();
            keys(&mut app, key);
            assert_eq!(app.vim.mode(), VimMode::Insert);
            assert_eq!(app.adding_parent, Some(parent));
            assert_eq!(app.input_priority, priority);
            keys(&mut app, "New child");
            enter(&mut app);
            assert_eq!(app.selected_todo().unwrap().parent_id, Some(parent));
            assert_eq!(app.selected_todo().unwrap().priority, priority);
        }
    }
}

#[test]
fn creation_keys_place_children_and_siblings_and_undo_keeps_order() {
    let mut app = app();
    for key in ["a", "i", "o", "O"] {
        keys(&mut app, key);
        assert_eq!(app.vim.mode(), VimMode::Insert);
        assert_eq!(app.adding_parent, None);
        assert_eq!(app.adding_relative, None);
        app.dispatch(VimAction::Cancel);
    }
    keys(&mut app, "iRoot");
    enter(&mut app);
    let root = app.selected_todo().unwrap().id;
    keys(&mut app, "aFirst child");
    assert_eq!(app.adding_parent, Some(root));
    enter(&mut app);
    let first = app.selected_todo().unwrap().id;
    keys(&mut app, "iGrandchild");
    assert_eq!(app.adding_parent, Some(first));
    enter(&mut app);
    keys(&mut app, "hoSecond child");
    enter(&mut app);
    keys(&mut app, "OBetween children");
    enter(&mut app);
    assert_eq!(app.selected_todo().unwrap().parent_id, Some(root));
    keys(&mut app, "ggoBelow root");
    enter(&mut app);
    app.select_id(root);
    keys(&mut app, "OAbove root");
    enter(&mut app);
    let selected = app.selected_todo().unwrap().id;
    app.reload(Some(selected)).unwrap();
    assert_eq!(
        app.visible_rows()
            .iter()
            .map(|row| { (app.todos[row.index].title.as_str(), row.depth) })
            .collect::<Vec<_>>(),
        [
            ("Above root", 0),
            ("Root", 0),
            ("First child", 1),
            ("Grandchild", 2),
            ("Between children", 1),
            ("Second child", 1),
            ("Below root", 0),
        ]
    );
    let expected = app.todos.clone();
    app.select_id(root);
    keys(&mut app, "O");
    app.dispatch(VimAction::Cancel);
    assert_eq!(app.selected_todo().unwrap().id, root);
    assert_eq!(app.todos, expected);
    keys(&mut app, "ddu");
    assert_eq!(app.todos, expected);
    keys(&mut app, "pho");
    assert_eq!(app.input_priority, Priority::High);
    keys(&mut app, "High sibling");
    enter(&mut app);
    assert_eq!(app.selected_todo().unwrap().priority, Priority::High);
    assert_eq!(app.selected_todo().unwrap().parent_id, None);
}

#[test]
fn priorities_sort_siblings_keep_selection_and_survive_edit_and_undo() {
    let mut app = app();
    keys(&mut app, "iFirst root");
    enter(&mut app);
    let first = app.selected_todo().unwrap().id;
    keys(&mut app, "oSecond root");
    enter(&mut app);
    let second = app.selected_todo().unwrap().id;
    for (priority, order) in [
        (Priority::High, [second, first]),
        (Priority::Low, [first, second]),
        (Priority::Mid, [first, second]),
    ] {
        keys(&mut app, "t");
        assert_eq!(app.selected_todo().unwrap().id, second);
        assert_eq!(app.selected_todo().unwrap().priority, priority);
        assert_eq!(
            app.visible_indices()
                .iter()
                .map(|&i| app.todos[i].id)
                .collect::<Vec<_>>(),
            order
        );
    }
    keys(&mut app, "phiMid child");
    enter(&mut app);
    keys(&mut app, "pm");
    let mid_child = app.selected_todo().unwrap().id;
    keys(&mut app, "oHigh child");
    cycle_input_priority(&mut app);
    enter(&mut app);
    let high_child = app.selected_todo().unwrap().id;
    assert_eq!(app.selected_todo().unwrap().priority, Priority::High);
    keys(&mut app, "iGrandchild");
    enter(&mut app);
    let grandchild = app.selected_todo().unwrap().id;
    keys(&mut app, "hpl");
    let rows = app.visible_rows();
    assert_eq!(
        rows.iter()
            .map(|row| (app.todos[row.index].id, row.depth))
            .collect::<Vec<_>>(),
        [
            (second, 0),
            (mid_child, 1),
            (high_child, 1),
            (grandchild, 2),
            (first, 0),
        ]
    );
    assert_eq!(app.selected_todo().unwrap().id, high_child);
    keys(&mut app, "e");
    cycle_input_priority(&mut app);
    app.dispatch(VimAction::Clear);
    keys(&mut app, "Updated child");
    enter(&mut app);
    assert_eq!(app.selected_todo().unwrap().priority, Priority::Mid);
    assert_eq!(app.selected_todo().unwrap().title, "Updated child");
    let expected = app.todos.clone();
    keys(&mut app, "e");
    cycle_input_priority(&mut app);
    app.dispatch(VimAction::Cancel);
    assert_eq!(app.todos, expected, "Cancel must not save priority changes");
    keys(&mut app, "ggdd");
    assert_eq!(app.todos.len(), 1);
    keys(&mut app, "u");
    assert_eq!(app.todos, expected);
    assert_eq!(app.selected_todo().unwrap().id, second);
}
#[test]
fn keyboard_workflow_keeps_completed_tasks_in_one_list() {
    let mut app = app();
    keys(&mut app, "iRead a book");
    enter(&mut app);
    keys(&mut app, "oShip the app");
    enter(&mut app);
    assert_eq!(app.selected_todo().unwrap().title, "Ship the app");
    keys(&mut app, "ke");
    app.dispatch(VimAction::Clear);
    keys(&mut app, "Read Rust");
    enter(&mut app);
    keys(&mut app, "x");
    assert_eq!(app.visible_indices().len(), 2);
    assert_eq!(app.selected_todo().unwrap().title, "Read Rust");
    assert!(app.selected_todo().unwrap().done);
    keys(&mut app, "j");
    assert_eq!(app.selected_todo().unwrap().title, "Ship the app");
    keys(&mut app, "kdd");
    assert_eq!(app.visible_indices().len(), 1);
    assert_eq!(app.selected_todo().unwrap().title, "Ship the app");
    keys(&mut app, "u");
    assert_eq!(app.selected_todo().unwrap().title, "Read Rust");
    assert!(app.selected_todo().unwrap().done);
    keys(&mut app, "2j");
    assert_eq!(app.selected_todo().unwrap().title, "Ship the app");
    keys(&mut app, "2k");
    assert_eq!(app.selected_todo().unwrap().title, "Read Rust");
    assert_eq!(app.todos.len(), 2);
}
#[test]
fn search_cancel_blank_title_and_empty_navigation() {
    let mut app = app();
    keys(&mut app, "jkGggdd3l");
    assert_eq!(app.list.selected(), None);
    assert_eq!(app.vim.mode(), VimMode::Normal);
    keys(&mut app, "i  ");
    enter(&mut app);
    assert!(app.error);
    assert_eq!(app.vim.mode(), VimMode::Insert);
    app.dispatch(VimAction::Cancel);
    keys(&mut app, "iBuy coffee");
    enter(&mut app);
    keys(&mut app, "oRead Rust");
    enter(&mut app);
    keys(&mut app, "/COFFEE");
    assert_eq!(app.visible_indices().len(), 1);
    app.dispatch(VimAction::Cancel);
    assert_eq!(app.visible_indices().len(), 2);
    keys(&mut app, "/rust");
    enter(&mut app);
    assert_eq!(app.selected_todo().unwrap().title, "Read Rust");
    app.dispatch(VimAction::Cancel);
    keys(&mut app, "gg2dd");
    assert_eq!(app.todos.len(), 0);
    keys(&mut app, "u");
    assert_eq!(app.todos.len(), 2);
}

#[test]
fn nested_tasks_stay_in_one_view_and_undo_restores_tree() {
    let mut app = app();
    keys(&mut app, "iProject");
    enter(&mut app);
    let root = app.selected_todo().unwrap().id;
    keys(&mut app, "i");
    assert_eq!(app.adding_parent, Some(root));
    assert_eq!(app.visible_indices().len(), 1);
    keys(&mut app, "Child");
    enter(&mut app);
    let child = app.selected_todo().unwrap().id;
    keys(&mut app, "iGrandchild");
    enter(&mut app);
    let grandchild = app.selected_todo().unwrap().id;
    keys(&mut app, "3l");
    assert_eq!(app.vim.mode(), VimMode::Normal);
    assert_eq!(app.selected_todo().unwrap().id, grandchild);
    assert_eq!(app.todos.len(), 3);
    keys(&mut app, "oSibling");
    enter(&mut app);
    assert_eq!(app.selected_todo().unwrap().parent_id, Some(child));
    assert_eq!(
        app.visible_rows()
            .iter()
            .map(|row| row.depth)
            .collect::<Vec<_>>(),
        [0, 1, 2, 2]
    );
    keys(&mut app, "2h");
    assert_eq!(app.selected_todo().unwrap().id, root);
    assert_eq!(app.visible_indices().len(), 4);
    keys(&mut app, "ll");
    assert_eq!(app.selected_todo().unwrap().id, grandchild);
    assert_eq!(app.visible_indices().len(), 4);
    keys(&mut app, "x");
    let expected = app.todos.clone();
    keys(&mut app, "/grand");
    enter(&mut app);
    let rows = app.visible_rows();
    assert_eq!(
        rows.iter()
            .map(|row| app.todos[row.index].id)
            .collect::<Vec<_>>(),
        [root, child, grandchild]
    );
    assert_eq!(
        rows.iter().map(|row| row.depth).collect::<Vec<_>>(),
        [0, 1, 2]
    );
    app.dispatch(VimAction::Cancel);
    keys(&mut app, "gghdd");
    assert!(app.todos.is_empty());
    keys(&mut app, "u");
    assert_eq!(app.todos, expected);
    assert_eq!(app.selected_todo().unwrap().id, root);
    assert_eq!(app.visible_indices().len(), 4);
    // Toggling a parent also reaches descendants hidden by search.
    keys(&mut app, "/Project");
    enter(&mut app);
    assert_eq!(app.visible_indices().len(), 1);
    keys(&mut app, "x");
    assert_eq!(app.selected_todo().unwrap().id, root);
    assert!(app.todos.iter().all(|todo| todo.done));
    assert!(app.database.list().unwrap().iter().all(|todo| todo.done));
    app.dispatch(VimAction::Cancel);
    keys(&mut app, "x");
    assert!(app.todos.iter().all(|todo| !todo.done));
}

#[test]
fn tree_order_groups_children_and_cancel_keeps_parent_selected() {
    let mut app = app();
    keys(&mut app, "iRoot");
    enter(&mut app);
    let root = app.selected_todo().unwrap().clone();
    keys(&mut app, "oOther root");
    enter(&mut app);
    keys(&mut app, "ki");
    app.dispatch(VimAction::Cancel);
    assert_eq!(app.selected_todo().unwrap().id, root.id);
    assert_eq!(app.adding_parent, None);
    keys(&mut app, "iChild");
    enter(&mut app);
    let rows = app.visible_rows();
    assert_eq!(
        rows.iter()
            .map(|row| app.todos[row.index].title.as_str())
            .collect::<Vec<_>>(),
        ["Root", "Child", "Other root"]
    );
    assert_eq!(
        rows.iter().map(|row| row.depth).collect::<Vec<_>>(),
        [0, 1, 0]
    );
    let child = app.selected_todo().unwrap().id;
    keys(&mut app, "/Child");
    enter(&mut app);
    keys(&mut app, "i");
    assert!(app.query.is_empty());
    assert_eq!(app.adding_parent, Some(child));
    assert_eq!(app.selected_todo().unwrap().id, child);
    assert_eq!(app.todos.len(), 3, "Drafts are not saved before Enter");
    app.dispatch(VimAction::Cancel);
    assert_eq!(app.selected_todo().unwrap().id, child);
    keys(&mut app, "jh");
    assert_eq!(app.selected_todo().unwrap().title, "Other root");
    app.database.delete(&[root]).unwrap();
    app.apply(VimAction::Refresh).unwrap();
    assert_eq!(app.selected_todo().unwrap().title, "Other root");
    assert_eq!(app.visible_indices().len(), 1);
}

#[test]
fn normal_and_visual_input_commands_save_titles_and_keep_task_actions_isolated() {
    let mut app = app();
    keys(&mut app, "iOne two three four");
    escape(&mut app);
    assert_eq!(app.vim.mode(), VimMode::Normal);
    assert_eq!(app.editor.text(), "One two three four");
    assert!(app.todos.is_empty());
    keys(&mut app, "03dw");
    assert_eq!(app.editor.text(), "four");
    assert!(app.todos.is_empty());
    enter(&mut app);
    assert_eq!(app.selected_todo().unwrap().title, "four");
    let saved = app.todos.clone();
    keys(&mut app, "e");
    escape(&mut app);
    keys(&mut app, "0viwr2");
    assert_eq!(app.editor.text(), "2222");
    assert_eq!(app.todos, saved);
    keys(&mut app, "q/?");
    assert!(app.running && !app.help);
    assert_eq!(app.todos, saved);
    escape(&mut app);
    assert!(app.vim.input().is_none());
    assert_eq!(app.todos, saved);

    keys(&mut app, "e");
    escape(&mut app);
    keys(&mut app, "dd");
    enter(&mut app);
    assert!(app.error && app.vim.input().is_some());
    assert_eq!(app.todos, saved);
    keys(&mut app, "3iX");
    enter(&mut app);
    assert_eq!(app.selected_todo().unwrap().title, "XXX");
}

#[test]
fn search_remains_live_in_normal_and_visual_modes_and_cancel_restores_query() {
    let mut app = app();
    keys(&mut app, "iBuy coffee");
    enter(&mut app);
    keys(&mut app, "oRead Rust");
    enter(&mut app);
    let saved = app.todos.clone();
    keys(&mut app, "/coffee");
    escape(&mut app);
    assert_eq!(app.visible_indices().len(), 1);
    keys(&mut app, "0viwcRust");
    assert_eq!(app.selected_todo().unwrap().title, "Read Rust");
    escape(&mut app);
    enter(&mut app);
    assert_eq!(app.query, "Rust");
    keys(&mut app, "/");
    escape(&mut app);
    keys(&mut app, "dd");
    assert_eq!(app.visible_indices().len(), 2);
    escape(&mut app);
    assert_eq!(app.query, "Rust");
    assert_eq!(app.visible_indices().len(), 1);
    assert_eq!(app.todos, saved);
}

#[test]
fn configuration_field_supports_normal_visual_undo_and_submit_from_visual() {
    let folder = std::env::temp_dir().join(format!(
        "argv-todo-modal-config-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let config = Config::load(&folder.join("custom.sql"), true).unwrap();
    let mut app = app();
    app.config = Some(config);
    escape(&mut app);
    enter(&mut app);
    escape(&mut app);
    keys(&mut app, "dd");
    enter(&mut app);
    assert!(app.error && app.vim.input().is_some());
    keys(&mut app, "u");
    assert_eq!(app.editor.text(), "db.sql");
    keys(&mut app, "0viwcstorage");
    escape(&mut app);
    assert_eq!(app.editor.text(), "storage.sql");
    keys(&mut app, "0V");
    enter(&mut app);
    assert!(app.configuring && app.vim.input().is_none());
    assert_eq!(app.config.as_ref().unwrap().database_path, "storage.sql");
    assert!(!folder.join("storage.sql").exists());
    assert!(app.todos.is_empty());
    std::fs::remove_dir_all(folder).unwrap();
}
