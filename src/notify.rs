//! Deadline reminders: due selection and the TUI-to-daemon push protocol.
//!
//! The TUI pushes its open deadlines to `argv-todo-daemon` over loopback TCP.
//! Messages are `v1` versioned lines of tab-separated fields with `\` escapes,
//! so titles survive arbitrary text without a serialization dependency.

use std::{
    collections::HashSet,
    fs,
    io::Write,
    net::TcpStream,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result};

use crate::{
    config::Config,
    db::{Priority, Todo, days_from_civil, now, split_deadline},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reminder {
    pub id: i64,
    pub title: String,
    pub deadline: String,
    pub priority: Priority,
}

/// Current UTC time as `YYYY-MM-DD HH:MM` for due comparisons.
pub fn current_time() -> String {
    now()
}

/// Open tasks with deadlines, earliest first.
pub fn upcoming(todos: &[Todo]) -> Vec<Reminder> {
    let mut reminders: Vec<_> = todos
        .iter()
        .filter(|todo| !todo.done && todo.deadline.is_some())
        .map(|todo| Reminder {
            id: todo.id,
            title: todo.title.clone(),
            deadline: todo.deadline.clone().unwrap_or_default(),
            priority: todo.priority,
        })
        .collect();
    reminders.sort_by(|a, b| a.deadline.cmp(&b.deadline).then_with(|| a.id.cmp(&b.id)));
    reminders
}

/// Which of the three notifications per deadline fired.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NotifySlot {
    Reminder1,
    Reminder2,
    Deadline,
}

/// Desktop urgency from the minutes left, bumped one level for high priority.
pub fn urgency_for(minutes_left: i64, priority: Priority) -> notify_rust::Urgency {
    let urgency = if minutes_left <= 0 {
        notify_rust::Urgency::Critical
    } else if minutes_left < 1440 {
        notify_rust::Urgency::Normal
    } else {
        notify_rust::Urgency::Low
    };
    if priority != Priority::High {
        return urgency;
    }
    match urgency {
        notify_rust::Urgency::Low => notify_rust::Urgency::Normal,
        _ => notify_rust::Urgency::Critical,
    }
}

/// Minutes since 1970-01-01 for a stored deadline, if it parses.
pub fn deadline_to_minutes(deadline: &str) -> Option<i64> {
    let (year, month, day, hour, minute) = split_deadline(deadline)?;
    Some(days_from_civil(year, month, day) * 1440 + i64::from(hour) * 60 + i64::from(minute))
}

/// Reminders with a notify moment (`deadline - offset`, then the deadline
/// itself) at or past now and not seen yet, earliest moment first.
/// Unparsable deadlines never notify. A zero offset disables that reminder.
pub fn select_due(
    reminders: &[Reminder],
    current: &str,
    first_minutes: u32,
    second_minutes: u32,
    seen: &HashSet<(i64, String, NotifySlot)>,
) -> Vec<(Reminder, NotifySlot, notify_rust::Urgency)> {
    let Some(current) = deadline_to_minutes(current) else {
        return Vec::new();
    };
    let mut due = Vec::new();
    for reminder in reminders {
        let Some(at) = deadline_to_minutes(&reminder.deadline) else {
            continue;
        };
        for (slot, offset) in [
            (NotifySlot::Reminder1, i64::from(first_minutes)),
            (NotifySlot::Reminder2, i64::from(second_minutes)),
            (NotifySlot::Deadline, 0),
        ] {
            if slot != NotifySlot::Deadline && offset == 0 {
                continue;
            }
            if seen.contains(&(reminder.id, reminder.deadline.clone(), slot)) {
                continue;
            }
            let notify_at = at - offset;
            if notify_at <= current {
                let left = at - current;
                due.push((reminder.clone(), slot, urgency_for(left, reminder.priority)));
            }
        }
    }
    due.sort_by(|a, b| {
        let a_at = deadline_to_minutes(&a.0.deadline).unwrap_or(i64::MAX);
        let b_at = deadline_to_minutes(&b.0.deadline).unwrap_or(i64::MAX);
        (a_at, a.1, a.0.id).cmp(&(b_at, b.1, b.0.id))
    });
    due
}

/// Port file living beside `config.toml`, written by the running daemon.
pub fn port_file_for(config_path: &Path) -> PathBuf {
    config_path.with_file_name("argv-todo-daemon.port")
}

/// Best-effort schedule push. Callers ignore the result: a missing daemon is
/// the normal case, not an error.
pub(crate) fn push_schedule(config: &Config, todos: &[Todo]) {
    if !config.notifications {
        return;
    }
    let _ = send_schedule(&config.path, &upcoming(todos));
}

fn send_schedule(config_path: &Path, reminders: &[Reminder]) -> Result<()> {
    let port = fs::read_to_string(port_file_for(config_path))
        .context("no daemon port file")?
        .trim()
        .parse::<u16>()
        .context("invalid daemon port")?;
    let mut stream = TcpStream::connect(("127.0.0.1", port)).context("daemon unreachable")?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .context("daemon write timeout")?;
    stream
        .write_all(encode_schedule(reminders).as_bytes())
        .context("daemon write failed")?;
    Ok(())
}

pub(crate) fn encode_schedule(reminders: &[Reminder]) -> String {
    let mut out = String::from("v1\n");
    for reminder in reminders {
        out.push_str(&reminder.id.to_string());
        out.push('\t');
        out.push_str(&reminder.deadline);
        out.push('\t');
        out.push_str(match reminder.priority {
            Priority::High => "high",
            Priority::Mid => "mid",
            Priority::Low => "low",
        });
        out.push('\t');
        out.push_str(&escape(&reminder.title));
        out.push('\n');
    }
    out
}

pub fn decode_schedule(text: &str) -> Option<Vec<Reminder>> {
    let mut lines = text.lines();
    if lines.next()? != "v1" {
        return None;
    }
    let mut reminders = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let mut fields = line.splitn(4, '\t');
        let id = fields.next()?;
        let deadline = fields.next()?;
        let priority = fields.next()?;
        let title = fields.next()?;
        reminders.push(Reminder {
            id: id.parse().ok()?,
            title: unescape(title)?,
            deadline: deadline.to_owned(),
            priority: match priority {
                "high" => Priority::High,
                "low" => Priority::Low,
                _ => Priority::Mid,
            },
        });
    }
    Some(reminders)
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out
}

fn unescape(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next()? {
            '\\' => out.push('\\'),
            't' => out.push('\t'),
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            _ => return None,
        }
    }
    Some(out)
}

#[cfg(test)]
#[path = "../tests/unit/notify.rs"]
mod tests;
