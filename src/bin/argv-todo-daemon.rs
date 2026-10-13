//! Standalone deadline notifier. Run it in the background next to the TUI:
//! `argv-todo-daemon &`. It uses the same `--db`/config files, seeds its
//! schedule from SQLite once, then waits for schedule pushes from `argv-todo`
//! over loopback TCP. A second invocation exits while one is already running.

use std::{
    collections::HashSet,
    fs,
    io::Read,
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, RecvTimeoutError},
    thread,
    time::Duration,
};

use anyhow::{Context, Result};
use argv_todo::{
    cli,
    config::Config,
    db::Database,
    notify::{self, NotifySlot, Reminder},
};

fn main() -> Result<()> {
    let Some(options) = cli::options()? else {
        return Ok(());
    };
    let config = Config::load(&options.database_path, options.database_override)?;
    let port_file = notify::port_file_for(&config.path);
    if let Ok(text) = fs::read_to_string(&port_file)
        && let Ok(port) = text.trim().parse::<u16>()
        && TcpStream::connect(("127.0.0.1", port)).is_ok()
    {
        anyhow::bail!("argv-todo-daemon is already running (port {port})");
    }
    let listener = TcpListener::bind("127.0.0.1:0").context("could not bind loopback")?;
    let port = listener.local_addr()?.port();
    fs::write(&port_file, port.to_string())
        .with_context(|| format!("could not write {}", port_file.display()))?;
    println!(
        "argv-todo-daemon started (port {port}, database {})",
        config.active_database.display()
    );
    let database = Database::open(&config.active_database).with_context(|| {
        format!(
            "Could not open database {}",
            config.active_database.display()
        )
    })?;
    let mut schedule = notify::upcoming(&database.list()?);
    let mut seen: HashSet<(i64, String, NotifySlot)> = HashSet::new();
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            if let Some(schedule) = read_schedule(stream)
                && sender.send(schedule).is_err()
            {
                break;
            }
        }
    });
    loop {
        // Settings apply without restarting: re-read them on every wake.
        let (first, second, enabled) =
            Config::load(&options.database_path, options.database_override)
                .map(|config| {
                    (
                        config.notify_reminder_1_minutes,
                        config.notify_reminder_2_minutes,
                        config.notifications,
                    )
                })
                .unwrap_or((60, 1440, true));
        let current = notify::current_time();
        if enabled {
            for (reminder, slot, urgency) in
                notify::select_due(&schedule, &current, first, second, &seen)
            {
                send_notification(&reminder, urgency);
                seen.insert((reminder.id, reminder.deadline.clone(), slot));
            }
        }
        match receiver.recv_timeout(time_until_next(&schedule, &current, first, second, &seen)) {
            Ok(updated) => schedule = updated,
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}

fn read_schedule(stream: TcpStream) -> Option<Vec<Reminder>> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut text = String::new();
    std::io::BufReader::new(stream)
        .read_to_string(&mut text)
        .ok()?;
    notify::decode_schedule(&text)
}

/// How long to sleep until the next unseen notify moment, capped at a day.
fn time_until_next(
    schedule: &[Reminder],
    current: &str,
    first_minutes: u32,
    second_minutes: u32,
    seen: &HashSet<(i64, String, NotifySlot)>,
) -> Duration {
    let Some(current) = notify::deadline_to_minutes(current) else {
        return Duration::from_secs(3600);
    };
    let offsets = [i64::from(first_minutes), i64::from(second_minutes), 0];
    let next = schedule
        .iter()
        .filter_map(|reminder| {
            let at = notify::deadline_to_minutes(&reminder.deadline)?;
            [
                (NotifySlot::Reminder1, offsets[0]),
                (NotifySlot::Reminder2, offsets[1]),
                (NotifySlot::Deadline, offsets[2]),
            ]
            .into_iter()
            .filter(|(slot, offset)| {
                (*slot == NotifySlot::Deadline || *offset > 0)
                    && !seen.contains(&(reminder.id, reminder.deadline.clone(), *slot))
            })
            .map(|(_, offset)| at - offset)
            .filter(|at| *at > current)
            .min()
        })
        .min();
    match next {
        Some(at) => Duration::from_secs(((at - current) * 60).max(1) as u64),
        None => Duration::from_secs(24 * 3600),
    }
}

fn send_notification(reminder: &Reminder, urgency: notify_rust::Urgency) {
    let result = notify_rust::Notification::new()
        .summary("Task deadline")
        .body(&format!("{} — due {}", reminder.title, reminder.deadline))
        .urgency(urgency)
        .show();
    if let Err(error) = result {
        eprintln!("argv-todo-daemon: {error:#}");
    }
}
