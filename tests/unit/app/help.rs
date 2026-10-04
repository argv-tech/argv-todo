use super::*;
use crate::db::Database;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

fn press(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
}

#[test]
fn section_selection_and_page_scrolling_keep_input_on_the_guide() {
    let mut app = App::new(Database::memory()).unwrap();
    app.help = true;
    app.help_view.set_sections(Some(7));
    app.help_view.set_viewport(40, 10);
    press(&mut app, KeyCode::Char('3'));
    press(&mut app, KeyCode::Char('j'));
    assert_eq!(app.help_view.selected_section(), 3);
    assert_eq!(app.help_view.offset(), 0);
    let mut key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
    key.kind = KeyEventKind::Repeat;
    app.handle_key(key);
    assert_eq!(app.help_view.selected_section(), 4);
    key.kind = KeyEventKind::Release;
    app.handle_key(key);
    assert_eq!(app.help_view.selected_section(), 4);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.help_view.selected_section(), 4);
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.help_view.selected_section(), 5);
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        let mut tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        tab.kind = kind;
        app.handle_key(tab);
        assert_eq!(app.help_view.selected_section(), 5);
    }
    press(&mut app, KeyCode::BackTab);
    assert_eq!(app.help_view.selected_section(), 4);
    press(&mut app, KeyCode::PageDown);
    assert_eq!(app.help_view.offset(), 9);
    assert_eq!(app.help_view.selected_section(), 4);
    press(&mut app, KeyCode::Char('k'));
    assert_eq!(app.help_view.selected_section(), 3);
    assert_eq!(app.help_view.offset(), 0);
    press(&mut app, KeyCode::End);
    press(&mut app, KeyCode::Char('j'));
    assert_eq!(app.help_view.selected_section(), 6);
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.help_view.selected_section(), 0);
    press(&mut app, KeyCode::BackTab);
    assert_eq!(app.help_view.selected_section(), 6);
    press(&mut app, KeyCode::Home);
    press(&mut app, KeyCode::Char('k'));
    assert_eq!(app.help_view.selected_section(), 0);
    assert!(app.todos.is_empty() && app.vim.input().is_none());
    assert!(!app.configuring);
    press(&mut app, KeyCode::Esc);
    assert!(!app.help);
}

#[test]
fn compact_help_scrolls_and_preserves_section_selection_when_guide_returns() {
    let mut app = App::new(Database::memory()).unwrap();
    app.help = true;
    app.help_view.set_sections(Some(7));
    app.help_view.set_viewport(30, 10);
    press(&mut app, KeyCode::Down);
    app.help_view.set_sections(None);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.help_view.offset(), 1);
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.help_view.selected_section(), 1);
    press(&mut app, KeyCode::Right);
    assert_eq!(app.help_view.offset(), 2);
    press(&mut app, KeyCode::Left);
    assert_eq!(app.help_view.offset(), 1);
    app.help_view.set_sections(Some(7));
    assert_eq!(app.help_view.selected_section(), 1);
}
