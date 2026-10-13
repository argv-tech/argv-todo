use crate::ui::tests::{terminal_row, text_position};
use crate::{
    app::App,
    config::{Config, SortOrder, TaskView},
    db::{Database, Priority},
    ui::{draw, theme::ACCENT},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Terminal, TerminalOptions, Viewport, backend::TestBackend, layout::Rect, style::Modifier,
};

fn app() -> App {
    let db = Database::memory();
    db.add("Inbox task", None, Priority::Mid).unwrap();
    for name in [
        "Work",
        "Personal",
        "Long project 界é👩‍💻 name",
        "Last project",
    ] {
        let project = db.create_project(name).unwrap();
        let id = db
            .add_project_root(&format!("Task for {name}"), Priority::Mid, Some(project))
            .unwrap();
        db.update_note(id, &format!("Note for {name}")).unwrap();
    }
    let mut app = App::new(db).unwrap();
    app.config = Some(Config {
        path: "/unused/config.toml".into(),
        database_path: "db.sql".into(),
        active_database: "/unused/db.sql".into(),
        database_override: false,
        task_view: TaskView::Projects,
        default_priority: Priority::Mid,
        show_completed: true,
        sort_order: SortOrder::Priority,
        show_hints: true,
        completed_delete: crate::db::DeleteMode::Soft,
    });
    app
}

#[test]
fn tabs_keep_active_project_visible_with_styles_and_scoped_notes_on_resize() {
    let mut app = app();
    let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
    for (width, height) in [(100, 24), (35, 12), (80, 16)] {
        terminal.backend_mut().resize(width, height);
        for _ in 0..5 {
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            let (_, name) = app.project_tabs().nth(app.active_project_index()).unwrap();
            let label = format!(
                "[{}]",
                crate::ui::text::truncate(name, usize::from(width).saturating_sub(8).min(22))
            );
            let prefix = label.split("界").next().unwrap();
            let (x, y) = text_position(&terminal, prefix);
            let end = x + unicode_width::UnicodeWidthStr::width(label.as_str()) as u16 - 1;
            assert_eq!(terminal.backend().buffer()[(end, y)].symbol(), "]");
            let cell = &terminal.backend().buffer()[(x, y)];
            assert_eq!(cell.fg, ACCENT);
            assert!(
                cell.modifier
                    .contains(Modifier::BOLD | Modifier::UNDERLINED)
            );
            text_position(&terminal, "0/1 done");
            if name == "Work" {
                text_position(&terminal, "Note for Work");
            }
            app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        }
    }
}

#[test]
fn project_editor_cursor_and_picker_overlay_work_with_nonzero_origin_and_tiny_frames() {
    let mut app = app();
    let mut terminal = Terminal::with_options(
        TestBackend::new(110, 38),
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(7, 4, 90, 28)),
        },
    )
    .unwrap();
    app.handle_key(KeyEvent::new(KeyCode::Char('P'), KeyModifiers::NONE));
    app.editor.insert("界é👩‍💻");
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let (x, y) = text_position(&terminal, "Project › ");
    let cursor = terminal.get_cursor_position().unwrap();
    assert_eq!((cursor.x, cursor.y), (x + 15, y));
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE));
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let (_, title_y) = text_position(&terminal, "Move task tree to project");
    assert!(title_y >= 4);
    text_position(&terminal, "Enter move");
    assert!(terminal_row(&terminal, title_y).contains('┌'));
    for (width, height) in [(35, 12), (1, 1), (0, 0), (80, 24)] {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        if width >= 35 {
            text_position(&terminal, "Move task tree to project");
            text_position(&terminal, "Enter move");
            text_position(&terminal, "Esc");
        }
    }
}
