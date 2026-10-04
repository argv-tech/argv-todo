use crate::{
    app::App,
    db::Database,
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

#[test]
fn help_groups_and_aligns_shortcuts_with_distinct_styles() {
    let mut app = App::new(Database::memory()).unwrap();
    app.help = true;
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "Keyboard shortcuts");
    let heading = text_position(&terminal, "TASKS / Navigation");
    let key = text_position(&terminal, "j / k");
    let description = text_position(&terminal, "Next / previous task");
    assert_eq!(description, (key.0 + 20, key.1));
    text_position(&terminal, "TASKS / Create & change");
    text_position(&terminal, "Esc / ? close");
    text_position(&terminal, "PgUp/PgDn page");
    text_position(&terminal, &format!("1-19/{}", app.help_view.total_rows()));
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer[key].fg, Color::Cyan);
    assert!(buffer[key].modifier.contains(Modifier::BOLD));
    assert_eq!(buffer[description].fg, Color::Reset);
    assert!(!buffer[description].modifier.contains(Modifier::BOLD));
    assert!(
        buffer[heading]
            .modifier
            .contains(Modifier::BOLD | Modifier::UNDERLINED)
    );
    assert!(
        buffer
            .content
            .iter()
            .any(|cell| cell.symbol() == "█" && cell.fg == Color::Cyan)
    );
    assert!(!terminal.backend().cursor_visible());
}

#[test]
fn help_wraps_on_narrow_screens_and_clamps_after_resize() {
    let mut app = App::new(Database::memory()).unwrap();
    app.help = true;
    let mut terminal = Terminal::new(TestBackend::new(35, 12)).unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let key = text_position(&terminal, "j / k");
    let description = text_position(&terminal, "Next / previous task");
    assert_eq!(description, (key.0 + 2, key.1 + 1));
    for (width, height) in [(35, 12), (80, 24), (120, 40), (35, 12)] {
        terminal.backend_mut().resize(width, height);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        app.handle_key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        text_position(&terminal, "Esc / ? close");
        text_position(&terminal, "swap case in Visual");
        let bottom = app.help_view.offset();
        app.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        assert_eq!(app.help_view.offset(), bottom);
        let footer_y = text_position(&terminal, "j/k scroll").1;
        assert!(terminal_row(&terminal, footer_y - 1).contains("Visual"));
        let (_, close_y) = text_position(&terminal, "Esc / ? close");
        assert_eq!(close_y, footer_y + 1);
    }
}

#[test]
fn help_tolerates_tiny_screens_and_nonzero_origins() {
    let mut app = App::new(Database::memory()).unwrap();
    app.help = true;
    for (width, height) in [(1, 1), (2, 2), (10, 5), (20, 8), (34, 11)] {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        assert!(!terminal.backend().cursor_visible());
        if width >= 20 {
            text_position(&terminal, "Esc / ? close");
        }
    }
    let area = Rect::new(7, 4, 80, 24);
    let mut terminal = Terminal::with_options(
        TestBackend::new(100, 35),
        TerminalOptions {
            viewport: Viewport::Fixed(area),
        },
    )
    .unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "Keyboard shortcuts");
    let buffer = terminal.backend().buffer();
    for y in 0..35 {
        for x in 0..100 {
            if !area.contains((x, y).into()) {
                assert_eq!(buffer[(x, y)].symbol(), " ");
            }
        }
    }
}
