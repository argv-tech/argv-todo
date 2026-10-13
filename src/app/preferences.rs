use super::App;
use crate::{
    config::{SortOrder, TaskView},
    db::{DEFAULT_PRIORITY, Priority},
};

impl App {
    pub(crate) fn default_priority(&self) -> Priority {
        self.config
            .as_ref()
            .map_or(DEFAULT_PRIORITY, |config| config.default_priority)
    }

    pub(crate) fn show_completed(&self) -> bool {
        self.task_view() != TaskView::Split
            && self
                .config
                .as_ref()
                .is_none_or(|config| config.show_completed)
    }

    pub(crate) fn sort_order(&self) -> SortOrder {
        self.config
            .as_ref()
            .map_or(SortOrder::Priority, |config| config.sort_order)
    }

    pub(crate) fn show_hints(&self) -> bool {
        self.config.as_ref().is_none_or(|config| config.show_hints)
    }

    pub(crate) fn notifications(&self) -> bool {
        self.config
            .as_ref()
            .is_none_or(|config| config.notifications)
    }

    pub(crate) fn notify_reminder_1_minutes(&self) -> u32 {
        self.config
            .as_ref()
            .map_or(60, |config| config.notify_reminder_1_minutes)
    }

    pub(crate) fn notify_reminder_2_minutes(&self) -> u32 {
        self.config
            .as_ref()
            .map_or(1440, |config| config.notify_reminder_2_minutes)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/app/preferences.rs"]
mod tests;
