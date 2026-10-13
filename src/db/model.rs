use rusqlite::{
    ToSql,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, Value, ValueRef},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    High = 1,
    #[default]
    Mid = 3,
    Low = 5,
}

impl Priority {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Mid => "mid",
            Self::Low => "low",
        }
    }

    pub(crate) fn next(self) -> Self {
        match self {
            Self::Low => Self::Mid,
            Self::Mid => Self::High,
            Self::High => Self::Low,
        }
    }
}

impl ToSql for Priority {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Integer(*self as i64)))
    }
}

impl FromSql for Priority {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            1 => Ok(Self::High),
            3 => Ok(Self::Mid),
            5 => Ok(Self::Low),
            value => Err(FromSqlError::OutOfRange(value)),
        }
    }
}

pub(crate) const DEFAULT_PRIORITY: Priority = Priority::Mid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) note: String,
    pub(crate) done: bool,
    pub(crate) deleted: bool,
    pub(crate) parent_id: Option<i64>,
    pub(crate) project_id: Option<i64>,
    pub(crate) priority: Priority,
    pub(crate) deadline: Option<String>,
    pub(crate) position: i64,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

impl Todo {
    pub(crate) fn urgency(&self, now: &str) -> Urgency {
        let (Some(deadline), false) = (self.deadline.as_deref(), self.done) else {
            return Urgency::Later;
        };
        let Some((year, month, day, hour, minute)) = split_deadline(deadline) else {
            return Urgency::Later;
        };
        let Some((now_year, now_month, now_day, now_hour, now_minute)) = split_deadline(now) else {
            return Urgency::Later;
        };
        // Zero-padded datetimes also compare chronologically as strings, but the
        // color bands below need the actual distance in minutes.
        let distance =
            (days_from_civil(year, month, day) * 1440 + i64::from(hour) * 60 + i64::from(minute))
                - (days_from_civil(now_year, now_month, now_day) * 1440
                    + i64::from(now_hour) * 60
                    + i64::from(now_minute));
        if distance < 0 {
            Urgency::Overdue
        } else if distance < 1440 {
            Urgency::Day
        } else if distance < 7 * 1440 {
            Urgency::Week
        } else {
            Urgency::Later
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Urgency {
    Overdue,
    Day,
    Week,
    Later,
}

pub(crate) fn is_leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

pub(crate) fn month_len(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if is_leap(year) {
                29
            } else {
                28
            }
        }
    }
}

/// Days since 1970-01-01 for a civil date; inverse of the conversion in [`today`].
pub(crate) const fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let adjust = if month <= 2 { 1 } else { 0 };
    let year = (year as i64) - adjust;
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_part = ((month as i64) + 9) % 12;
    let day_of_year = (153 * month_part + 2) / 5 + (day as i64) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Split a stored deadline into numbers. Accepts `YYYY-MM-DD HH:MM` and,
/// for databases predating the time picker, bare `YYYY-MM-DD` as midnight.
pub(crate) fn split_deadline(text: &str) -> Option<(i32, u32, u32, u32, u32)> {
    if text.len() == 10 {
        let (year, month, day) = split_iso(text)?;
        return Some((year, month, day, 0, 0));
    }
    if text.len() != 16 || text.as_bytes()[10] != b' ' {
        return None;
    }
    let (year, month, day) = split_iso(&text[..10])?;
    let digits = |start: usize, end: usize| {
        text[start..end]
            .bytes()
            .all(|byte| byte.is_ascii_digit())
            .then(|| text[start..end].parse::<u32>().ok())
            .flatten()
    };
    let hour = digits(11, 13)?;
    let minute = digits(14, 16)?;
    if hour > 23 || minute > 59 {
        return None;
    }
    Some((year, month, day, hour, minute))
}

fn split_iso(text: &str) -> Option<(i32, u32, u32)> {
    let bytes = text.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let digits = |start: usize, end: usize| {
        text[start..end]
            .bytes()
            .all(|byte| byte.is_ascii_digit())
            .then(|| text[start..end].parse::<u32>().ok())
            .flatten()
    };
    let year = digits(0, 4)? as i32;
    let month = digits(5, 7)?;
    let day = digits(8, 10)?;
    if month == 0 || month > 12 || day == 0 || day > month_len(year, month) {
        return None;
    }
    Some((year, month, day))
}

/// Today's UTC date as `YYYY-MM-DD`, matching SQLite `date('now')`.
pub(crate) fn today() -> String {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() / 86400)
        .unwrap_or(0) as i64;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Current UTC time as `YYYY-MM-DD HH:MM` for deadline storage and comparison.
pub(crate) fn now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    let (year, month, day) = civil_from_days((secs / 86400) as i64);
    let hour = (secs % 86400) / 3600;
    let minute = (secs % 3600) / 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}")
}

/// Civil date for days since 1970-01-01 without new dependencies.
pub(crate) fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_part = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_part + 2) / 5 + 1;
    let month = if month_part < 10 {
        month_part + 3
    } else {
        month_part - 9
    };
    (
        (year + i64::from(month <= 2)) as i32,
        month as u32,
        day as u32,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Project {
    pub(crate) id: i64,
    pub(crate) name: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeleteMode {
    #[default]
    Soft,
    Hard,
}

impl DeleteMode {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Soft => "soft",
            Self::Hard => "hard",
        }
    }

    pub(crate) fn next(self) -> Self {
        match self {
            Self::Soft => Self::Hard,
            Self::Hard => Self::Soft,
        }
    }
}
