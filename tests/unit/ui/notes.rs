use crate::{
    app::App,
    config::{Config, TaskView},
    db::{Database, Priority},
    ui::{
        draw,
        tests::{terminal_row, text_position},
    },
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Terminal, TerminalOptions, Viewport,
    backend::TestBackend,
    layout::Rect,
    style::{Color, Modifier},
};

fn press(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
}

fn app(view: TaskView) -> App {
    let mut database = Database::memory();
    let root = database.add("Parent", None, Priority::Mid).unwrap();
    let child = database.add("Child", Some(root), Priority::Mid).unwrap();
    database.update_note(root, "Parent note 界é👩‍💻").unwrap();
    database.update_note(child, "Child note").unwrap();
    if view == TaskView::Split {
        database.toggle(child).unwrap();
    }
    let mut app = App::new(database).unwrap();
    app.config = Some(Config {
        path: "/unused/config.toml".into(),
        database_path: "db.sql".into(),
        active_database: "/unused/db.sql".into(),
        database_override: false,
        task_view: view,
        default_priority: Priority::Mid,
        show_completed: true,
        sort_order: crate::config::SortOrder::Priority,
        show_hints: true,
    });
    app
}

#[test]
fn bottom_note_follows_selection_and_focused_pane_after_resize() {
    for view in [TaskView::Normal, TaskView::Split] {
        let mut app = app(view);
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        for (width, height) in [(80, 24), (35, 12), (120, 35)] {
            terminal.backend_mut().resize(width, height);
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            let heading = text_position(&terminal, "Note · n edit");
            let note = text_position(&terminal, "Parent note");
            let task = text_position(&terminal, "Parent");
            assert!(task.1 < heading.1 && heading.1 < note.1 && note.1 < height - 2);
            assert_eq!(terminal.backend().buffer()[note].fg, Color::Reset);
            assert_eq!(
                terminal.backend().buffer()[(note.0 + 12, note.1)].symbol(),
                "界"
            );
            assert_eq!(
                terminal.backend().buffer()[(note.0 + 14, note.1)].symbol(),
                "é"
            );
            assert_eq!(
                terminal.backend().buffer()[(note.0 + 15, note.1)].symbol(),
                "👩‍💻"
            );
            assert!(
                terminal.backend().buffer()[heading]
                    .modifier
                    .contains(Modifier::BOLD)
            );
            assert!(!terminal.backend().cursor_visible());
        }
        press(
            &mut app,
            if view == TaskView::Split {
                KeyCode::Tab
            } else {
                KeyCode::Char('j')
            },
        );
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        text_position(&terminal, "Child note");
        let screen = (0..35)
            .map(|y| terminal_row(&terminal, y))
            .collect::<String>();
        assert!(!screen.contains("Parent note"));
        press(&mut app, KeyCode::Char('n'));
        assert_eq!(app.editor.text(), "Child note");
        app.editor.insert(" saved");
        press(&mut app, KeyCode::Enter);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        text_position(&terminal, "Child note saved");
    }
}

#[test]
fn note_wraps_and_editor_cursor_stays_inside_nonzero_and_tiny_areas() {
    let mut app = app(TaskView::Normal);
    app.todos[0].note = "wrap-note".repeat(20);
    let area = Rect::new(7, 4, 80, 24);
    let mut terminal = Terminal::with_options(
        TestBackend::new(100, 35),
        TerminalOptions {
            viewport: Viewport::Fixed(area),
        },
    )
    .unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let heading = text_position(&terminal, "Note · n edit");
    let text = (heading.1 + 1..area.bottom() - 2)
        .map(|y| terminal_row(&terminal, y).trim().to_owned())
        .collect::<String>();
    assert_eq!(text, app.todos[0].note);
    app.todos[0].note = "界é👩‍💻".repeat(40);
    press(&mut app, KeyCode::Char('n'));
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let focused = text_position(&terminal, "› Note");
    assert_eq!(terminal.backend().buffer()[focused].fg, Color::Cyan);
    assert!(terminal.backend().cursor_visible());
    let cursor = terminal.backend().cursor_position();
    assert!(area.contains(cursor));
    assert_eq!(cursor.y, focused.1 + 1);

    let mut terminal = Terminal::new(TestBackend::new(35, 12)).unwrap();
    for (width, height) in [(35, 12), (1, 1), (0, 0), (80, 24)] {
        terminal.backend_mut().resize(width, height);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        if width >= 35 {
            assert!(terminal.backend().cursor_visible());
            let cursor = terminal.backend().cursor_position();
            assert!(terminal.backend().buffer().area.contains(cursor));
            assert_eq!(cursor.y, text_position(&terminal, "› Note").1 + 1);
        } else {
            assert!(!terminal.backend().cursor_visible());
        }
    }
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Esc);
    app.todos[0].note.clear();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "No note yet.");
    assert!(!terminal.backend().cursor_visible());
    press(&mut app, KeyCode::Char('/'));
    app.editor.insert("No match");
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "Select a task to see its note.");
}
