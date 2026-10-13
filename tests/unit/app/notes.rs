use super::*;
use crate::{
    db::{Database, Priority},
    vim_motion::{VimAction, VimMode},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn press(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
}

fn save(app: &mut App) {
    app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
}

#[test]
fn notes_save_to_selected_tasks_and_can_be_cleared_or_cancelled() {
    let database = Database::memory();
    let parent = database.add("Parent", None, Priority::High).unwrap();
    let child = database.add("Child", Some(parent), Priority::Low).unwrap();
    let mut app = App::new(database).unwrap();
    let original = app.todos.clone();
    press(&mut app, KeyCode::Char('n'));
    assert_eq!(app.vim.input(), Some(InputTarget::Note));
    app.handle_paste("Parent's note 界é👩‍💻\r\nwith paste");
    assert_eq!(app.editor.text(), "Parent's note 界é👩‍💻\nwith paste");
    // Task commands stay in the field and never affect the selection or priority.
    app.handle_key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL));
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.selected_todo().unwrap().id, parent);
    assert_eq!(app.todos, original);
    save(&mut app);
    assert!(app.vim.input().is_none());
    assert_eq!(
        app.database.list().unwrap()[0].note,
        "Parent's note 界é👩‍💻\nwith paste"
    );
    assert_eq!(app.todos[0].title, original[0].title);
    assert_eq!(app.todos[0].priority, original[0].priority);
    assert!(app.todos[1].note.is_empty());

    press(&mut app, KeyCode::Char('j'));
    press(&mut app, KeyCode::Char('n'));
    app.handle_paste("Child note");
    save(&mut app);
    assert_eq!(app.selected_todo().unwrap().id, child);
    assert_eq!(app.todos[1].note, "Child note");
    let saved = app.todos.clone();
    press(&mut app, KeyCode::Char('n'));
    app.handle_paste("discard");
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.vim.mode(), VimMode::Normal);
    assert_eq!(app.vim.input(), Some(InputTarget::Note));
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.todos, saved);
    assert_eq!(app.database.list().unwrap(), saved);
    assert!(app.vim.input().is_none() && app.editing_id.is_none());
    assert!(!app.configuring);

    press(&mut app, KeyCode::Char('n'));
    app.dispatch(VimAction::Clear);
    save(&mut app);
    assert!(app.todos[1].note.is_empty());
    assert_eq!(app.todos[0].note, saved[0].note);
    assert!(!app.error);
}

#[test]
fn missing_task_keeps_note_draft_available_after_save_error() {
    let database = Database::memory();
    database.add("Task", None, Priority::Mid).unwrap();
    let mut app = App::new(database).unwrap();
    press(&mut app, KeyCode::Char('n'));
    app.handle_paste("Keep this draft");
    app.database.delete(&app.todos).unwrap();
    save(&mut app);
    assert!(app.error);
    assert!(app.status.contains("Task no longer exists"));
    assert_eq!(app.vim.input(), Some(InputTarget::Note));
    assert_eq!(app.editor.text(), "Keep this draft");
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Esc);
    app.dispatch(VimAction::Refresh);
    assert!(app.todos.is_empty());
    press(&mut app, KeyCode::Char('n'));
    assert!(app.vim.input().is_none());
    assert!(!app.error);
}

#[test]
fn multiline_notes_preserve_spacing_blank_lines_and_save_after_restart() {
    let folder = std::env::temp_dir().join(format!("argv-multiline-note-{}", std::process::id()));
    let path = folder.join("db.sql");
    let database = Database::open(&path).unwrap();
    database.add("Task", None, Priority::Mid).unwrap();
    let mut app = App::new(database).unwrap();
    press(&mut app, KeyCode::Char('n'));
    app.handle_paste("  first 界é👩‍💻\r\n\r\nsecond");
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.vim.input(), Some(InputTarget::Note));
    app.handle_paste("    third\n");
    let expected = "  first 界é👩‍💻\n\nsecond\n    third\n";
    assert_eq!(app.editor.text(), expected);
    save(&mut app);
    assert!(!app.error && app.vim.input().is_none());
    assert_eq!(app.todos[0].note, expected);
    drop(app);
    let mut app = App::new(Database::open(&path).unwrap()).unwrap();
    press(&mut app, KeyCode::Char('n'));
    assert_eq!(app.editor.text(), expected);
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Enter);
    assert!(app.vim.input().is_none());
    assert_eq!(app.todos[0].note, expected);
    drop(app);
    std::fs::remove_dir_all(folder).unwrap();
}
