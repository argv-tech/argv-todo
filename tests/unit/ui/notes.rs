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
        completed_delete: crate::db::DeleteMode::Soft,
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
            let heading = text_position(&terminal, "Note · Parent");
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
        app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        text_position(&terminal, "Child note saved");
        text_position(&terminal, "Note · Child");
        if view == TaskView::Split {
            text_position(&terminal, "Todo  1");
            text_position(&terminal, "› Completed  1");
        }
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
    let heading = text_position(&terminal, "Note · Parent");
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

#[test]
fn note_height_follows_content_and_clipped_previews_end_with_an_ellipsis() {
    use super::note_height;

    let mut app = app(TaskView::Normal);
    let area = Rect::new(2, 3, 76, 19);
    assert_eq!(note_height(&app, area), 2);
    app.todos[0].note = "long preview words ".repeat(80);
    assert_eq!(note_height(&app, area), 5);
    let saved = app.todos.clone();
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let heading = text_position(&terminal, "Note · Parent");
    assert_eq!(heading.1, 17);
    assert!(terminal_row(&terminal, 21).trim_end().ends_with('…'));
    assert_eq!(app.todos, saved);

    app.todos[0].note = "Short note".into();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    assert_eq!(text_position(&terminal, "Note · Parent").1, 20);
    text_position(&terminal, "Short note");
    assert!(!(0..24).any(|y| terminal_row(&terminal, y).contains("long preview")));

    assert_eq!(note_height(&app, Rect::default()), 0);
    app.config.as_mut().unwrap().task_view = TaskView::Split;
    // Search and errors get priority over a secondary preview in a short stacked layout.
    assert_eq!(note_height(&app, Rect::new(1, 3, 33, 6)), 0);
    press(&mut app, KeyCode::Char('n'));
    assert_eq!(note_height(&app, Rect::new(1, 3, 33, 6)), 2);
}

#[test]
fn note_heading_truncates_task_context_and_respects_hidden_hints() {
    let mut app = app(TaskView::Normal);
    app.todos[0].title = "界é👩‍💻".repeat(30);
    let mut terminal = Terminal::new(TestBackend::new(35, 12)).unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let heading = text_position(&terminal, "Note ·");
    let hint = text_position(&terminal, "n edit");
    assert_eq!(heading.1, hint.1);
    assert!(terminal_row(&terminal, heading.1).contains('…'));
    assert_eq!(terminal.backend().buffer()[hint].fg, Color::DarkGray);
    let saved = app.todos.clone();
    app.config.as_mut().unwrap().show_hints = false;
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    assert!(!(0..12).any(|y| terminal_row(&terminal, y).contains("n edit")));
    press(&mut app, KeyCode::Char('n'));
    app.editor.reset(String::new());
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let focused = text_position(&terminal, "› Note");
    let placeholder = text_position(&terminal, "Write a note…");
    assert_eq!(terminal.backend().buffer()[focused].fg, Color::Cyan);
    assert_eq!(terminal.backend().buffer()[placeholder].fg, Color::DarkGray);
    assert_eq!(terminal.backend().cursor_position(), placeholder.into());
    assert!(terminal.backend().cursor_visible());
    assert!(!(0..12).any(|y| terminal_row(&terminal, y).contains("Enter save")));
    assert_eq!(app.todos, saved);
}

#[test]
fn note_rendering_clips_to_small_nonzero_areas_and_keeps_the_editor_cursor_inside() {
    use super::draw_note;
    let mut app = app(TaskView::Normal);
    press(&mut app, KeyCode::Char('n'));
    app.editor.insert(&"界é👩‍💻".repeat(20));
    let saved = app.todos.clone();
    for (width, height) in [(0, 0), (1, 1), (2, 2), (3, 2), (15, 3), (76, 5)] {
        let area = Rect::new(7, 4, width, height);
        let mut terminal = Terminal::new(TestBackend::new(100, 35)).unwrap();
        terminal.draw(|frame| draw_note(frame, &app, area)).unwrap();
        for y in 0..35 {
            for x in 0..100 {
                if !area.contains((x, y).into()) {
                    assert_eq!(terminal.backend().buffer()[(x, y)].symbol(), " ");
                }
            }
        }
        if terminal.backend().cursor_visible() {
            assert!(area.contains(terminal.backend().cursor_position()));
        }
        assert_eq!(terminal.backend().cursor_visible(), width > 2 && height > 1);
        assert_eq!(app.todos, saved);
    }
}

#[test]
fn multiline_note_editor_keeps_rows_cursor_and_selection_visible_on_resize() {
    let mut app = app(TaskView::Normal);
    app.todos[0].note = "first\n\n界é👩‍💻 last\nend\n".into();
    press(&mut app, KeyCode::Char('n'));
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let heading = text_position(&terminal, "› Note");
    assert_eq!(terminal.backend().cursor_position().y, heading.1 + 5);
    text_position(&terminal, "Ctrl-s save");
    app.handle_key(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let end = text_position(&terminal, "end");
    assert_eq!(terminal.backend().cursor_position(), end.into());
    app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Char('V'));
    press(&mut app, KeyCode::Char('k'));
    for (width, height) in [(80, 24), (35, 12), (120, 35), (1, 1), (0, 0)] {
        terminal.backend_mut().resize(width, height);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        if width < 35 {
            assert!(!terminal.backend().cursor_visible());
            continue;
        }
        let blank_cursor = terminal.backend().cursor_position();
        assert!(terminal.backend().buffer().area.contains(blank_cursor));
        assert!(
            terminal.backend().buffer()[blank_cursor]
                .modifier
                .contains(Modifier::REVERSED)
        );
        text_position(&terminal, "v-line");
    }
    let area = Rect::new(7, 4, 35, 12);
    let mut terminal = Terminal::with_options(
        TestBackend::new(55, 25),
        TerminalOptions {
            viewport: Viewport::Fixed(area),
        },
    )
    .unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    assert!(area.contains(terminal.backend().cursor_position()));
    for y in 0..25 {
        for x in 0..55 {
            if !area.contains((x, y).into()) {
                assert_eq!(terminal.backend().buffer()[(x, y)].symbol(), " ");
            }
        }
    }
}

#[test]
fn multiline_previews_preserve_blank_rows_and_note_height_handles_tiny_areas() {
    let mut app = app(TaskView::Normal);
    app.todos[0].note = "first\n\nthird\n".into();
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let first = text_position(&terminal, "first");
    let third = text_position(&terminal, "third");
    assert_eq!(third.1, first.1 + 2);
    assert!(terminal_row(&terminal, first.1 + 1).trim().is_empty());
    press(&mut app, KeyCode::Char('n'));
    for (width, height) in [(1, 1), (2, 2), (0, 0)] {
        assert!(super::note_height(&app, Rect::new(7, 4, width, height)) <= height);
    }
}
