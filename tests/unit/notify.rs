use super::*;
use crate::db::{Database, Priority};
use std::collections::HashSet;
use std::io::Read;
use std::net::TcpListener;

fn reminders() -> Vec<Reminder> {
    let mut db = Database::memory();
    let past = db.add("Past \"quoted\" task", None, Priority::Mid).unwrap();
    db.set_deadline(past, Some("2026-10-20 11:30")).unwrap();
    let soon = db.add("Soon\ttab task", None, Priority::Mid).unwrap();
    db.set_deadline(soon, Some("2026-10-20 12:10")).unwrap();
    let done = db.add("Done", None, Priority::Mid).unwrap();
    db.set_deadline(done, Some("2026-10-20 11:00")).unwrap();
    db.toggle(done).unwrap();
    let plain = db.add("No deadline", None, Priority::Mid).unwrap();
    let _ = plain;
    upcoming(&db.list().unwrap())
}

#[test]
fn upcoming_keeps_open_deadlines_earliest_first() {
    let upcoming = reminders();
    assert_eq!(
        upcoming
            .iter()
            .map(|reminder| reminder.title.as_str())
            .collect::<Vec<_>>(),
        ["Past \"quoted\" task", "Soon\ttab task"]
    );
    assert_eq!(upcoming[0].deadline, "2026-10-20 11:30");
}

#[test]
fn select_due_fires_reminders_deadline_and_seen_slots() {
    let upcoming = reminders();
    let seen = HashSet::new();
    // At noon with 60/1440 reminders: the 11:30 task owes all three slots,
    // the 12:10 task owes both reminders but not its deadline yet.
    let due = select_due(&upcoming, "2026-10-20 12:00", 60, 1440, &seen);
    assert_eq!(
        due.iter()
            .map(|(reminder, slot, _)| (reminder.deadline.as_str(), *slot))
            .collect::<Vec<_>>(),
        [
            ("2026-10-20 11:30", NotifySlot::Reminder1),
            ("2026-10-20 11:30", NotifySlot::Reminder2),
            ("2026-10-20 11:30", NotifySlot::Deadline),
            ("2026-10-20 12:10", NotifySlot::Reminder1),
            ("2026-10-20 12:10", NotifySlot::Reminder2),
        ]
    );
    // Urgency follows the minutes left: overdue is critical, ten minutes Normal.
    assert_eq!(due[0].2, notify_rust::Urgency::Critical);
    assert_eq!(due[3].2, notify_rust::Urgency::Normal);
    // A zero offset disables that reminder; the deadline still fires.
    let due = select_due(&upcoming, "2026-10-20 12:00", 0, 0, &seen);
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].1, NotifySlot::Deadline);
    // Seen slots never repeat; later slots still fire on their own moment.
    let mut seen = HashSet::new();
    for (reminder, slot, _) in select_due(&upcoming, "2026-10-20 12:00", 60, 1440, &seen) {
        seen.insert((reminder.id, reminder.deadline.clone(), slot));
    }
    let due = select_due(&upcoming, "2026-10-20 23:59", 60, 1440, &seen);
    assert_eq!(
        due.iter()
            .map(|(reminder, slot, _)| (reminder.deadline.as_str(), *slot))
            .collect::<Vec<_>>(),
        [("2026-10-20 12:10", NotifySlot::Deadline)]
    );
    // Garbage deadlines and garbage clocks never notify.
    let mut broken = upcoming.clone();
    broken.push(Reminder {
        id: 99,
        title: "Broken".into(),
        deadline: "not a deadline".into(),
        priority: Priority::Mid,
    });
    assert_eq!(
        select_due(&broken, "2026-10-20 12:00", 60, 1440, &seen).len(),
        0
    );
    assert!(select_due(&upcoming, "not a clock", 60, 1440, &seen).is_empty());
    assert_eq!(deadline_to_minutes("1970-01-01 00:00"), Some(0));
    assert_eq!(deadline_to_minutes("1970-01-01 01:30"), Some(90));
    assert_eq!(deadline_to_minutes("not a deadline"), None);
}

#[test]
fn urgency_follows_time_left_and_boosts_high_priority() {
    use notify_rust::Urgency::{Critical, Low, Normal};
    assert_eq!(urgency_for(-1, Priority::Mid), Critical);
    assert_eq!(urgency_for(0, Priority::Low), Critical);
    assert_eq!(urgency_for(10, Priority::Mid), Normal);
    assert_eq!(urgency_for(1439, Priority::Mid), Normal);
    assert_eq!(urgency_for(1440, Priority::Mid), Low);
    assert_eq!(urgency_for(10, Priority::High), Critical);
    assert_eq!(urgency_for(2000, Priority::High), Normal);
    assert_eq!(urgency_for(2000, Priority::Low), Low);
}

#[test]
fn schedule_protocol_round_trips_arbitrary_titles() {
    let upcoming = reminders();
    let decoded = decode_schedule(&encode_schedule(&upcoming)).unwrap();
    assert_eq!(decoded, upcoming);
    assert_eq!(decode_schedule("v0\n1\t2026-10-20 12:00\tmid\tHi\n"), None);
    assert_eq!(decode_schedule("v1\nno-tabs-here\n"), None);
    assert_eq!(
        decode_schedule("v1\nNaN\t2026-10-20 12:00\tmid\tHi\n"),
        None
    );
    assert_eq!(
        decode_schedule("v1\n1\t2026-10-20 12:00\tmid\tbad\\escape\\q\n"),
        None
    );
    // Unknown priority labels fall back to mid instead of dropping the row.
    assert_eq!(
        decode_schedule("v1\n1\t2026-10-20 12:00\turgent\tHi\n").unwrap()[0].priority,
        Priority::Mid
    );
    // Blank lines are tolerated; the version line is mandatory.
    assert_eq!(decode_schedule("v1\n\n"), Some(Vec::new()));
    assert_eq!(decode_schedule(""), None);
}

#[test]
fn task_priority_survives_the_push_protocol() {
    // Priorities ride the push protocol so the daemon can boost urgency.
    let upcoming = reminders();
    let decoded = decode_schedule(&encode_schedule(&upcoming)).unwrap();
    assert_eq!(
        decoded
            .iter()
            .map(|reminder| reminder.priority)
            .collect::<Vec<_>>(),
        [Priority::Mid, Priority::Mid]
    );
}
#[test]
fn port_file_sits_beside_the_config() {
    assert_eq!(
        port_file_for(std::path::Path::new("/tmp/x/config.toml")),
        std::path::PathBuf::from("/tmp/x/argv-todo-daemon.port")
    );
}

#[test]
fn push_schedule_skips_disabled_config_and_missing_daemon() {
    let folder = std::env::temp_dir().join(format!(
        "argv-notify-push-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let config = Config::load(&folder.join("db.sql"), false).unwrap();
    // Fresh configs enable notifications, but no daemon listens: still silent.
    push_schedule(&config, &[]);
    let mut disabled = Config::load(&folder.join("other.sql"), false).unwrap();
    disabled.notifications = false;
    push_schedule(&disabled, &[]);
    assert!(!port_file_for(&config.path).exists());
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn schedule_push_reaches_a_listening_daemon() {
    let folder = std::env::temp_dir().join(format!(
        "argv-notify-socket-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&folder).unwrap();
    let config_path = folder.join("config.toml");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    std::fs::write(
        folder.join("argv-todo-daemon.port"),
        listener.local_addr().unwrap().port().to_string(),
    )
    .unwrap();
    let handle = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
        let mut text = String::new();
        std::io::BufReader::new(stream)
            .read_to_string(&mut text)
            .unwrap();
        decode_schedule(&text).unwrap()
    });
    let reminders = vec![
        Reminder {
            id: 7,
            title: "Hi".into(),
            deadline: "2026-10-20 12:00".into(),
            priority: Priority::High,
        },
        Reminder {
            id: 3,
            title: "Earlier".into(),
            deadline: "2026-10-19 09:00".into(),
            priority: Priority::Low,
        },
    ];
    send_schedule(&config_path, &reminders).unwrap();
    assert_eq!(handle.join().unwrap(), reminders);
    // The listener is gone, so further pushes fail for the caller to ignore.
    assert!(send_schedule(&config_path, &reminders).is_err());
    std::fs::remove_dir_all(folder).unwrap();
}
