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
    let mut terminal = Terminal::new(TestBackend::new(120, 32)).unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    text_position(&terminal, "Keyboard shortcuts");
    text_position(&terminal, "SHORTCUT GUIDE");
    let guide = text_position(&terminal, "› TASKS / NAVIGATION");
    let heading = (guide.0 + 2, guide.1);
    let key = text_position(&terminal, "j / k");
    let description = text_position(&terminal, "Next / previous task");
    assert_eq!(description, (key.0 + 20, key.1));
    text_position(&terminal, "TASKS / CREATE & CHANGE");
    text_position(&terminal, "Esc / ? close");
    text_position(&terminal, "PgUp/PgDn scroll");
    let total = app.help_view.total_rows();
    text_position(&terminal, &format!("1-{total}/{total}"));
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
        (0..32).any(|y| buffer[(116, y)].symbol() == "█" && buffer[(116, y)].fg == Color::Cyan)
    );
    assert!(!terminal.backend().cursor_visible());
}

#[test]
fn help_matches_settings_branding_and_tracks_the_current_section() {
    let mut app = App::new(Database::memory()).unwrap();
    app.help = true;
    let mut terminal = Terminal::new(TestBackend::new(120, 32)).unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    for (row, logo) in super::super::branding::LOGO.iter().enumerate() {
        let position = text_position(&terminal, logo.trim_end());
        assert_eq!(position.1, 1 + row as u16);
        assert_eq!(terminal.backend().buffer()[position].fg, Color::Cyan);
    }
    let guide = text_position(&terminal, "SHORTCUT GUIDE");
    let document = super::content::document(80);
    for code in [
        KeyCode::Home,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Up,
        KeyCode::End,
        KeyCode::Home,
    ] {
        app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        let active = app.help_view.selected_section();
        for (index, section) in document.sections.iter().enumerate() {
            let y = guide.1 + 2 + index as u16 * 2;
            assert!(terminal_row(&terminal, y).contains(&section.title.to_uppercase()));
            let title = &terminal.backend().buffer()[(guide.0 + 2, y)];
            if index == active {
                assert!(
                    title
                        .modifier
                        .contains(Modifier::BOLD | Modifier::UNDERLINED)
                );
                assert_eq!(terminal.backend().buffer()[(guide.0, y)].symbol(), "›");
                assert_eq!(terminal.backend().buffer()[(guide.0, y)].fg, Color::Cyan);
            } else {
                assert_eq!(title.fg, Color::Reset);
                assert!(title.modifier.contains(Modifier::BOLD));
                assert!(!title.modifier.contains(Modifier::UNDERLINED));
            }
        }
    }
    for (width, height) in [(120, 32), (80, 24), (35, 12), (120, 12)] {
        terminal.backend_mut().resize(width, height);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        text_position(&terminal, "Esc / ? close");
        let screen = (0..height)
            .map(|y| terminal_row(&terminal, y))
            .collect::<Vec<_>>();
        assert_eq!(
            screen.iter().any(|row| row.contains("SHORTCUT GUIDE")),
            width >= 80 && height >= 24
        );
    }
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
        if width >= 80 {
            loop {
                let offset = app.help_view.offset();
                app.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
                terminal.draw(|frame| draw(frame, &mut app)).unwrap();
                if app.help_view.offset() == offset {
                    break;
                }
            }
        }
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        text_position(&terminal, "Esc / ? close");
        text_position(&terminal, "anywhere");
        let bottom = app.help_view.offset();
        app.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        assert_eq!(app.help_view.offset(), bottom);
        let footer_y = text_position(
            &terminal,
            if width >= 80 {
                "j/k section"
            } else {
                "j/k scroll"
            },
        )
        .1;
        if width == 35 || height == 24 {
            assert!(terminal_row(&terminal, footer_y - 1).contains("anywhere"));
        }
        let (_, close_y) = text_position(&terminal, "Esc / ? close");
        assert_eq!(close_y, footer_y + 1);
    }
}

#[test]
fn guide_selection_opens_one_section_and_keeps_its_title_visible() {
    let mut app = App::new(Database::memory()).unwrap();
    app.help = true;
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let body_y = text_position(&terminal, "SHORTCUT GUIDE").1;
    for _ in 0..2 {
        app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    }
    assert_eq!(app.help_view.selected_section(), 2);
    let right_title = (36..76)
        .map(|x| terminal.backend().buffer()[(x, body_y)].symbol())
        .collect::<String>();
    assert!(right_title.starts_with("APP / SEARCH & SETTINGS"));
    assert_eq!(terminal.backend().buffer()[(36, body_y)].fg, Color::Cyan);
    app.handle_key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    let title = text_position(&terminal, "› APP / SEARCH & SETTINGS");
    assert!(title.1 < text_position(&terminal, "Esc / ? close").1);
    assert_eq!(app.help_view.selected_section(), 2);
    app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    terminal.draw(|frame| draw(frame, &mut app)).unwrap();
    assert_eq!(app.help_view.selected_section(), 0);
    text_position(&terminal, "› TASKS / NAVIGATION");
    let right_title = (36..76)
        .map(|x| terminal.backend().buffer()[(x, body_y)].symbol())
        .collect::<String>();
    assert!(right_title.starts_with("TASKS / NAVIGATION"));
    assert!(!terminal.backend().cursor_visible());
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
