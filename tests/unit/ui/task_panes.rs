use crate::{
    app::{App, TaskPane},
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

fn app(view: TaskView) -> App {
    let mut database = Database::memory();
    let parent = database.add("Parent", None, Priority::Mid).unwrap();
    let child = database
        .add("Child 界é👩‍💻", Some(parent), Priority::Low)
        .unwrap();
    database
        .add("Grandchild", Some(child), Priority::Low)
        .unwrap();
    if view == TaskView::Split {
        database.toggle(child).unwrap();
    }
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
fn folded_rows_show_a_marker_and_expand_after_resize_without_losing_styles() {
    let mut app = app(TaskView::Normal);
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    press(&mut app, KeyCode::Enter);
    for (width, height) in [(80, 24), (35, 12), (1, 1), (100, 24)] {
        terminal.backend_mut().resize(width, height);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        if width >= 35 {
            let parent = text_position(&terminal, "Parent");
            assert!(terminal_row(&terminal, parent.1).contains("+ 0/1"));
            assert!(
                terminal.backend().buffer()[parent]
                    .modifier
                    .contains(Modifier::BOLD | Modifier::UNDERLINED)
            );
            let screen = (0..height)
                .map(|y| terminal_row(&terminal, y))
                .collect::<String>();
            assert!(!screen.contains("Child") && !screen.contains("Grandchild"));
            text_position(&terminal, "Other root");
        }
    }
    press(&mut app, KeyCode::Enter);
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let child = text_position(&terminal, "Child");
    assert!(terminal_row(&terminal, child.1).contains("界"));
    assert!(terminal_row(&terminal, child.1).contains("é"));
    assert!(terminal_row(&terminal, child.1).contains("👩‍💻"));
    text_position(&terminal, "Grandchild");
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Char('i'));
    app.editor.insert("Draft 界é👩‍💻");
    terminal.backend_mut().resize(35, 12);
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "Draft");
    assert!(terminal.backend().cursor_visible());
    let cursor = terminal.backend().cursor_position();
    assert!(cursor.x < 35 && cursor.y < 12);
}

#[test]
fn completed_pane_dims_ghost_parents_and_edits_the_real_child() {
    let mut app = app(TaskView::Split);
    let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
    press(&mut app, KeyCode::Tab);
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let heading = text_position(&terminal, "› Completed");
    let parent = (heading.0 + 10, heading.1 + 1);
    let child = (heading.0 + 15, heading.1 + 2);
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer[parent].symbol(), "P");
    assert_eq!(buffer[parent].fg, Color::DarkGray);
    assert!(buffer[parent].modifier.contains(Modifier::DIM));
    assert!(
        !buffer[parent]
            .modifier
            .intersects(Modifier::UNDERLINED | Modifier::CROSSED_OUT)
    );
    assert!(terminal_row(&terminal, parent.1).contains("(parent)"));
    assert!(
        buffer[child]
            .modifier
            .contains(Modifier::UNDERLINED | Modifier::CROSSED_OUT)
    );
    press(&mut app, KeyCode::Char('e'));
    app.editor.insert(&"界é👩‍💻".repeat(30));
    for (width, height) in [(100, 24), (35, 12), (74, 12)] {
        terminal.backend_mut().resize(width, height);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        let cursor = terminal.backend().cursor_position();
        assert!(terminal.backend().cursor_visible());
        assert!(cursor.x < width - 1 && cursor.y < height - 1);
        if width >= 74 {
            assert!(cursor.x > width / 2);
        }
    }
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.task_pane(), TaskPane::Completed);
    assert_eq!(
        app.todos
            .iter()
            .find(|todo| todo.title == "Parent")
            .unwrap()
            .title,
        "Parent"
    );
    assert!(
        app.todos
            .iter()
            .find(|todo| todo.title.starts_with("Child"))
            .unwrap()
            .title
            .contains("👩‍💻界")
    );
}

#[test]
fn split_remembers_each_panes_selection_and_scroll_offset() {
    let mut database = Database::memory();
    for row in 0..25 {
        database
            .add(&format!("Todo {row}"), None, Priority::Mid)
            .unwrap();
        let id = database
            .add(&format!("Done {row}"), None, Priority::Mid)
            .unwrap();
        database.toggle(id).unwrap();
    }
    let config = app(TaskView::Split).config;
    let mut app = App::new(database).unwrap();
    app.config = config;
    let mut terminal = Terminal::new(TestBackend::new(100, 16)).unwrap();
    press(&mut app, KeyCode::Char('G'));
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let todo_state = *app.pane_list(TaskPane::Todo);
    assert!(todo_state.offset() > 0);
    text_position(&terminal, "Todo 24");
    press(&mut app, KeyCode::Tab);
    press(&mut app, KeyCode::Char('G'));
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let completed_state = *app.pane_list(TaskPane::Completed);
    assert!(completed_state.offset() > 0);
    text_position(&terminal, "Done 24");
    press(&mut app, KeyCode::Tab);
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    assert_eq!(app.pane_list(TaskPane::Todo), &todo_state);
    assert_eq!(app.pane_list(TaskPane::Completed), &completed_state);
}

#[test]
fn split_layout_and_focus_styles_survive_resize() {
    let mut app = app(TaskView::Split);
    let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
    for (width, height) in [(100, 24), (74, 12), (73, 12), (35, 12), (120, 35)] {
        terminal.backend_mut().resize(width, height);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        let todo = text_position(&terminal, "› Todo");
        let completed = text_position(&terminal, "Completed");
        if width >= 74 {
            assert!(completed.0 > todo.0);
            assert_eq!(completed.1, todo.1);
        } else {
            assert_eq!(completed.0, todo.0 + 2);
            assert!(completed.1 > todo.1);
        }
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[todo].fg, Color::Cyan);
        assert!(buffer[todo].modifier.contains(Modifier::BOLD));
        assert_eq!(buffer[completed].fg, Color::DarkGray);
        let child_title = (
            completed.0 + 13,
            completed.1 + 2 - app.pane_list(TaskPane::Completed).offset() as u16,
        );
        assert_eq!(buffer[child_title].symbol(), "C");
        assert!(!buffer[child_title].modifier.contains(Modifier::UNDERLINED));
        press(&mut app, KeyCode::Tab);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        text_position(&terminal, "› Completed");
        let buffer = terminal.backend().buffer();
        assert!(
            buffer[child_title]
                .modifier
                .contains(Modifier::UNDERLINED | Modifier::BOLD)
        );
        assert_eq!(buffer[(child_title.0 - 5, child_title.1)].fg, Color::Blue);
        assert!(!terminal.backend().cursor_visible());
        press(&mut app, KeyCode::Tab);
    }
}

#[test]
fn split_handles_empty_data_tiny_areas_and_nonzero_origins() {
    let mut app = app(TaskView::Split);
    let area = Rect::new(7, 4, 80, 24);
    let mut terminal = Terminal::with_options(
        TestBackend::new(100, 32),
        TerminalOptions {
            viewport: Viewport::Fixed(area),
        },
    )
    .unwrap();
    press(&mut app, KeyCode::Tab);
    press(&mut app, KeyCode::Char('i'));
    app.editor.insert("Draft 界é👩‍💻");
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    assert!(area.contains(terminal.backend().cursor_position()));
    for y in 0..32 {
        for x in 0..100 {
            if !area.contains((x, y).into()) {
                assert_eq!(terminal.backend().buffer()[(x, y)].symbol(), " ");
            }
        }
    }
    let mut terminal = Terminal::new(TestBackend::new(35, 12)).unwrap();
    for (width, height) in [(0, 0), (1, 1), (34, 12), (35, 11), (35, 12)] {
        terminal.backend_mut().resize(width, height);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        assert_eq!(
            terminal.backend().cursor_visible(),
            width >= 35 && height >= 12
        );
    }
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Esc);
    app.todos.clear();
    terminal.backend_mut().resize(80, 24);
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "No unfinished todos.");
    text_position(&terminal, "No completed tasks.");
    assert!(!terminal.backend().cursor_visible());
    assert!((0..24).any(|y| terminal_row(&terminal, y).contains("Tab")));
}
