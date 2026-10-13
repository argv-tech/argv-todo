use crate::{
    app::App,
    db::{Database, DeleteMode, Priority, TaskScope},
    ui::{
        draw,
        tests::{terminal_row, text_position},
        theme::ACCENT,
    },
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Terminal, TerminalOptions, Viewport, backend::TestBackend, layout::Rect, style::Modifier,
};

fn archived_app() -> App {
    let mut db = Database::memory();
    let project = db.create_project("Work 界é👩‍💻").unwrap();
    for index in 0..30 {
        let id = db
            .add_project_root(
                &format!("Saved {index:02} 界é👩‍💻"),
                Priority::Mid,
                if index % 2 == 0 { Some(project) } else { None },
            )
            .unwrap();
        db.update_note(id, &format!("Note {index:02} 界é👩‍💻"))
            .unwrap();
        db.toggle(id).unwrap();
    }
    db.delete_completed(TaskScope::All, DeleteMode::Soft)
        .unwrap();
    db.add("Active task", None, Priority::Mid).unwrap();
    let mut app = App::new(db).unwrap();
    app.handle_key(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE));
    app
}

#[test]
fn archive_renders_notes_styles_and_selection_while_resizing() {
    let mut app = archived_app();
    let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
    for (width, height) in [(100, 24), (35, 12), (1, 1), (0, 0), (80, 18)] {
        terminal.backend_mut().resize(width, height);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        assert!(!terminal.backend().cursor_visible());
        if width >= 35 {
            let heading = text_position(&terminal, "Soft-deleted tasks");
            assert_eq!(terminal.backend().buffer()[heading].fg, ACCENT);
            assert!(
                terminal.backend().buffer()[heading]
                    .modifier
                    .contains(Modifier::BOLD)
            );
            let selected = text_position(&terminal, "Saved 00");
            assert!(
                terminal.backend().buffer()[selected]
                    .modifier
                    .contains(Modifier::BOLD | Modifier::UNDERLINED)
            );
            text_position(&terminal, "Note 00");
            text_position(&terminal, "A/Esc back");
            let screen = (0..height)
                .map(|y| terminal_row(&terminal, y))
                .collect::<String>();
            assert!(!screen.contains("Active task") && !screen.contains("n edit"));
            if width == 100 {
                assert_eq!(
                    terminal.backend().buffer()[(selected.0 + 9, selected.1)].symbol(),
                    "界"
                );
                assert_eq!(
                    terminal.backend().buffer()[(selected.0 + 11, selected.1)].symbol(),
                    "é"
                );
                assert_eq!(
                    terminal.backend().buffer()[(selected.0 + 12, selected.1)].symbol(),
                    "👩‍💻"
                );
                text_position(&terminal, "Work");
                text_position(&terminal, "Inbox");
            }
        }
    }
    app.handle_key(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE));
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "Saved 29");
    text_position(&terminal, "Note 29");
    assert!(app.archive.as_ref().unwrap().list.offset() > 0);
    let offset = app.archive.as_ref().unwrap().list.offset();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    assert_eq!(app.archive.as_ref().unwrap().list.offset(), offset);
}

#[test]
fn empty_archive_and_nonzero_origin_clip_safely_and_return_to_tasks() {
    let mut app = archived_app();
    let area = Rect::new(7, 4, 80, 24);
    let mut terminal = Terminal::with_options(
        TestBackend::new(100, 35),
        TerminalOptions {
            viewport: Viewport::Fixed(area),
        },
    )
    .unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    for y in 0..35 {
        for x in 0..100 {
            if !area.contains((x, y).into()) {
                assert_eq!(terminal.backend().buffer()[(x, y)].symbol(), " ");
            }
        }
    }
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "Active task");
    let mut app = App::new(Database::memory()).unwrap();
    app.handle_key(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE));
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "No soft-deleted tasks.");
    assert!(app.archive.as_ref().unwrap().list.selected().is_none());
    for (width, height) in [(0, 0), (1, 1), (2, 2), (35, 12)] {
        let small = Rect::new(7, 4, width, height);
        terminal
            .draw(|frame| super::draw_archive(frame, &mut app, small))
            .unwrap();
    }
}

#[test]
fn archive_restore_and_clear_prompt_keep_controls_visible_on_resize() {
    let mut app = archived_app();
    let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
    app.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE));
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "29 archived");
    text_position(&terminal, "Saved 01");
    text_position(&terminal, "Note 01");
    text_position(&terminal, "r restore tree");
    text_position(&terminal, "D clear all");
    app.handle_key(KeyEvent::new(KeyCode::Char('D'), KeyModifiers::NONE));
    for area in [
        Rect::new(0, 0, 100, 24),
        Rect::new(7, 4, 35, 12),
        Rect::new(7, 4, 1, 1),
        Rect::new(7, 4, 0, 0),
    ] {
        terminal
            .draw(|frame| super::draw_archive(frame, &mut app, area))
            .unwrap();
        if area.width < 35 {
            continue;
        }
        let heading = text_position(&terminal, "Clear archive permanently?");
        assert_eq!(terminal.backend().buffer()[heading].fg, ACCENT);
        assert!(
            terminal.backend().buffer()[heading]
                .modifier
                .contains(Modifier::BOLD)
        );
        let hint = text_position(&terminal, "Enter clears · Esc cancels");
        assert!(area.contains(hint.into()));
        text_position(&terminal, "29 tasks · all projects · no undo");
        assert!(!terminal.backend().cursor_visible());
    }
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "Soft-deleted tasks");
    app.handle_key(KeyEvent::new(KeyCode::Char('D'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "No soft-deleted tasks.");
}
