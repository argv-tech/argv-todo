use anyhow::Result;

use super::App;
use crate::{
    db::{civil_from_days, days_from_civil, month_len, now, split_deadline, today},
    vim_motion::{DateUnit, VimAction},
};

const MIN_DAYS: i64 = days_from_civil(1, 1, 1);
const MAX_DAYS: i64 = days_from_civil(9999, 12, 31);

/// Monday-first weekday (0) through Sunday (6) for civil days.
fn monday_first(days: i64) -> u32 {
    // 1970-01-01 was a Thursday.
    (days + 3).rem_euclid(7) as u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum PickerFocus {
    #[default]
    Date,
    Hour,
    Minute,
}

impl PickerFocus {
    fn cycle(self, steps: i32) -> Self {
        match (self as i32 + steps).rem_euclid(3) {
            1 => Self::Hour,
            2 => Self::Minute,
            _ => Self::Date,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DatePicker {
    pub(crate) task_id: i64,
    pub(crate) year: i32,
    pub(crate) month: u32,
    pub(crate) day: u32,
    pub(crate) hour: u32,
    pub(crate) minute: u32,
    pub(crate) focus: PickerFocus,
}

impl DatePicker {
    fn new(task_id: i64, year: i32, month: u32, day: u32, hour: u32, minute: u32) -> Self {
        Self {
            task_id,
            year,
            month,
            day,
            hour,
            minute,
            focus: PickerFocus::Date,
        }
    }

    pub(crate) fn to_iso(self) -> String {
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute
        )
    }

    pub(crate) fn set_today(&mut self) {
        let Some((year, month, day, _, _)) =
            split_deadline(&today()).or_else(|| split_deadline(&now()))
        else {
            return;
        };
        self.year = year;
        self.month = month;
        self.day = day;
    }

    pub(crate) fn shift(&mut self, unit: DateUnit, steps: i32) {
        match unit {
            DateUnit::Day => self.shift_days(i64::from(steps)),
            DateUnit::Week => self.shift_days(i64::from(steps) * 7),
            DateUnit::Month => {
                let total =
                    i64::from(self.year) * 12 + i64::from(self.month - 1) + i64::from(steps);
                // Year 1 January is month index 12; year 0 never exists.
                let total = total.clamp(12, 9999 * 12 + 11);
                self.year = total.div_euclid(12) as i32;
                self.month = total.rem_euclid(12) as u32 + 1;
                self.day = self.day.min(month_len(self.year, self.month));
            }
        }
    }

    fn shift_days(&mut self, delta: i64) {
        let days =
            (days_from_civil(self.year, self.month, self.day) + delta).clamp(MIN_DAYS, MAX_DAYS);
        let (year, month, day) = civil_from_days(days);
        self.year = year;
        self.month = month;
        self.day = day;
    }

    pub(crate) fn step_hour(&mut self, steps: i32) {
        self.hour = (self.hour as i32 + steps).rem_euclid(24) as u32;
    }

    pub(crate) fn step_minute(&mut self, steps: i32) {
        self.minute = (self.minute as i32 + steps).rem_euclid(60) as u32;
    }

    /// Monday-first weekday of the cursor (0) through Sunday (6).
    pub(crate) fn weekday_index(&self) -> u32 {
        monday_first(days_from_civil(self.year, self.month, self.day))
    }

    /// Blanks before the 1st of the month in a Sunday-first grid.
    pub(crate) fn leading_blanks(&self) -> u32 {
        (monday_first(days_from_civil(self.year, self.month, 1)) + 1) % 7
    }
}

impl App {
    pub(in crate::app) fn begin_date_picker(&mut self) {
        let Some(todo) = self.selected_todo() else {
            self.message("Select a task to set a deadline.");
            return;
        };
        let (year, month, day, hour, minute) = todo
            .deadline
            .as_deref()
            .and_then(split_deadline)
            .or_else(|| split_deadline(&now()))
            .unwrap_or((1970, 1, 1, 0, 0));
        self.date_picker = Some(DatePicker::new(todo.id, year, month, day, hour, minute));
        self.vim.end_input();
        self.message("");
    }

    pub(in crate::app) fn apply_date_picker(&mut self, action: VimAction) -> Result<()> {
        match action {
            VimAction::ShiftDate(unit, steps) => {
                let Some(picker) = self.date_picker.as_mut() else {
                    return Ok(());
                };
                match picker.focus {
                    PickerFocus::Date => picker.shift(unit, steps),
                    PickerFocus::Hour => match unit {
                        // H jumps to the start of the day, L to the end.
                        DateUnit::Month if steps < 0 => picker.hour = 0,
                        DateUnit::Month => picker.hour = 23,
                        DateUnit::Week => picker.step_hour(steps),
                        DateUnit::Day => picker.step_hour(steps),
                    },
                    PickerFocus::Minute => match unit {
                        DateUnit::Month if steps < 0 => {
                            picker.hour = 0;
                            picker.minute = 0;
                        }
                        DateUnit::Month => {
                            picker.hour = 23;
                            picker.minute = 59;
                        }
                        DateUnit::Week => picker.step_minute(steps * 5),
                        DateUnit::Day => picker.step_minute(steps),
                    },
                }
            }
            VimAction::FocusNext => {
                if let Some(picker) = self.date_picker.as_mut() {
                    picker.focus = picker.focus.cycle(1);
                }
            }
            VimAction::FocusPrev => {
                if let Some(picker) = self.date_picker.as_mut() {
                    picker.focus = picker.focus.cycle(-1);
                }
            }
            VimAction::PickToday => {
                if let Some(picker) = self.date_picker.as_mut() {
                    picker.set_today();
                }
            }
            VimAction::Cancel => {
                self.date_picker = None;
                self.vim.end_input();
                self.message("Deadline unchanged.");
            }
            VimAction::ClearDeadline => {
                let Some(picker) = self.date_picker.take() else {
                    return Ok(());
                };
                self.database.set_deadline(picker.task_id, None)?;
                self.vim.end_input();
                self.reload(Some(picker.task_id))?;
                self.message("Deadline cleared.");
            }
            VimAction::Submit => {
                let Some(picker) = self.date_picker.take() else {
                    return Ok(());
                };
                let deadline = picker.to_iso();
                self.database
                    .set_deadline(picker.task_id, Some(&deadline))?;
                self.vim.end_input();
                self.reload(Some(picker.task_id))?;
                self.message(format!("Deadline set to {deadline}."));
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/app/deadlines.rs"]
mod tests;
