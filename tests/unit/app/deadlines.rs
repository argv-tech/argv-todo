use super::*;
use crate::db::{Database, Priority, civil_from_days, days_from_civil, now, split_deadline};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn keys(app: &mut App, text: &str) {
    for c in text.chars() {
        app.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    }
}

fn key(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
}

fn escape(app: &mut App) {
    let action = app
        .vim
        .handle(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
        .unwrap();
    app.dispatch(action);
}

#[test]
fn picker_date_math_clamps_months_years_and_leap_days() {
    // Civil round-trips, including the Unix epoch anchor.
    assert_eq!(days_from_civil(1970, 1, 1), 0);
    assert_eq!(civil_from_days(0), (1970, 1, 1));
    assert_eq!(
        civil_from_days(days_from_civil(2026, 10, 20)),
        (2026, 10, 20)
    );
    assert_eq!(civil_from_days(days_from_civil(1, 1, 1)), (1, 1, 1));
    assert_eq!(
        civil_from_days(days_from_civil(9999, 12, 31)),
        (9999, 12, 31)
    );
    // October 2026 starts on a Thursday, so four Sunday-first blanks lead the grid.
    assert_eq!(DatePicker::new(1, 2026, 10, 20, 14, 30).leading_blanks(), 4);
    assert_eq!(
        DatePicker::new(1, 2026, 10, 20, 14, 30).to_iso(),
        "2026-10-20 14:30"
    );

    // Month steps clamp the day instead of overflowing into the next month.
    let mut picker = DatePicker::new(1, 2026, 1, 31, 0, 0);
    picker.shift(DateUnit::Month, 1);
    assert_eq!((picker.year, picker.month, picker.day), (2026, 2, 28));
    let mut picker = DatePicker::new(1, 2024, 1, 31, 0, 0);
    picker.shift(DateUnit::Month, 1);
    assert_eq!((picker.year, picker.month, picker.day), (2024, 2, 29));

    // Months roll over year boundaries.
    let mut picker = DatePicker::new(1, 2026, 12, 15, 0, 0);
    picker.shift(DateUnit::Month, 1);
    assert_eq!((picker.year, picker.month, picker.day), (2027, 1, 15));
    picker.shift(DateUnit::Month, -2);
    assert_eq!((picker.year, picker.month, picker.day), (2026, 11, 15));

    // Day and week steps cross month boundaries.
    let mut picker = DatePicker::new(1, 2026, 10, 31, 0, 0);
    picker.shift(DateUnit::Day, 1);
    assert_eq!(picker.to_iso(), "2026-11-01 00:00");
    picker.shift(DateUnit::Week, -1);
    assert_eq!(picker.to_iso(), "2026-10-25 00:00");
    picker.shift(DateUnit::Day, -25);
    assert_eq!(picker.to_iso(), "2026-09-30 00:00");

    // The cursor never leaves the supported year range.
    let mut picker = DatePicker::new(1, 1, 1, 1, 0, 0);
    picker.shift(DateUnit::Day, -1);
    picker.shift(DateUnit::Month, -1);
    assert_eq!(picker.to_iso(), "0001-01-01 00:00");
    let mut picker = DatePicker::new(1, 9999, 12, 31, 23, 59);
    picker.shift(DateUnit::Day, 1);
    picker.shift(DateUnit::Month, 1);
    assert_eq!(picker.to_iso(), "9999-12-31 23:59");

    // Hours and minutes wrap around the clock.
    let mut picker = DatePicker::new(1, 2026, 10, 20, 23, 59);
    picker.step_hour(1);
    picker.step_minute(1);
    assert_eq!(picker.to_iso(), "2026-10-20 00:00");
    picker.step_hour(-1);
    picker.step_minute(-1);
    assert_eq!(picker.to_iso(), "2026-10-20 23:59");

    // Stored datetimes seed the cursor; bare dates seed midnight for legacy rows.
    assert_eq!(
        split_deadline("2026-10-20 14:30"),
        Some((2026, 10, 20, 14, 30))
    );
    assert_eq!(split_deadline("2026-10-20"), Some((2026, 10, 20, 0, 0)));
    for text in [
        "2026-02-29 12:00",
        "2026-10-20 24:00",
        "2026-10-20 12:60",
        "2026-10-20 1:30",
        "2026-10-2012:00",
        "not a date",
    ] {
        assert_eq!(split_deadline(text), None, "{text:?}");
    }
    let (year, month, day, hour, minute) = split_deadline(&now()).unwrap();
    assert_eq!(
        format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}"),
        now()
    );
}

#[test]
fn picker_saves_clears_and_cancels_through_keys() {
    let db = Database::memory();
    db.add("Ship it", None, Priority::Mid).unwrap();
    let mut app = App::new(db).unwrap();
    assert!(app.date_picker.is_none());

    // Without a saved deadline the cursor starts on the current time.
    keys(&mut app, "T");
    let current = now();
    assert_eq!(
        app.date_picker.map(|picker| picker.to_iso()),
        Some(current.clone())
    );
    keys(&mut app, "l");
    let (year, month, day, hour, minute) = split_deadline(&current).unwrap();
    let expected = civil_from_days(days_from_civil(year, month, day) + 1);
    key(&mut app, KeyCode::Enter);
    assert!(app.date_picker.is_none());
    assert_eq!(
        app.todos[0].deadline,
        Some(format!(
            "{:04}-{:02}-{:02} {hour:02}:{minute:02}",
            expected.0, expected.1, expected.2
        ))
    );
    assert!(app.status.starts_with("Deadline set to "));

    // Reopening starts on the saved deadline; Esc keeps it.
    keys(&mut app, "T");
    assert_eq!(
        app.date_picker.map(|picker| picker.to_iso()),
        app.todos[0].deadline
    );
    keys(&mut app, "h");
    escape(&mut app);
    assert!(app.date_picker.is_none());
    assert!(app.status.contains("Deadline unchanged."));

    // `t` jumps back to today and keeps the picked time.
    keys(&mut app, "T");
    keys(&mut app, "H");
    assert_ne!(
        app.date_picker.map(|picker| picker.to_iso()),
        Some(current.clone())
    );
    keys(&mut app, "t");
    let cursor = app.date_picker.unwrap();
    let (year, month, day, _, _) = split_deadline(&current).unwrap();
    assert_eq!((cursor.year, cursor.month, cursor.day), (year, month, day));
    escape(&mut app);

    // Tab cycles through the time fields, which adjust independently.
    let (_, _, _, start_hour, start_minute) = split_deadline(&current).unwrap();
    keys(&mut app, "T");
    key(&mut app, KeyCode::Tab);
    assert_eq!(
        app.date_picker.map(|picker| picker.focus),
        Some(PickerFocus::Hour)
    );
    keys(&mut app, "l");
    assert_eq!(
        app.date_picker.map(|picker| picker.hour),
        Some((start_hour + 1) % 24)
    );
    key(&mut app, KeyCode::Tab);
    assert_eq!(
        app.date_picker.map(|picker| picker.focus),
        Some(PickerFocus::Minute)
    );
    keys(&mut app, "ll");
    assert_eq!(
        app.date_picker.map(|picker| picker.minute),
        Some((start_minute + 2) % 60)
    );
    key(&mut app, KeyCode::Tab);
    assert_eq!(
        app.date_picker.map(|picker| picker.focus),
        Some(PickerFocus::Date)
    );
    key(&mut app, KeyCode::Enter);
    let saved = app.todos[0].deadline.clone().unwrap();
    assert_ne!(saved, current);

    // `x` clears the saved deadline without touching the selection.
    keys(&mut app, "T");
    keys(&mut app, "x");
    assert!(app.date_picker.is_none());
    assert_eq!(app.todos[0].deadline, None);
    assert!(app.status.contains("Deadline cleared."));

    // Without a selection there is nothing to pick.
    keys(&mut app, "ggdd");
    key(&mut app, KeyCode::Enter);
    keys(&mut app, "T");
    assert!(app.date_picker.is_none());
    assert!(app.status.contains("Select a task"));
}
