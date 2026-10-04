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
        self.task_view() == TaskView::Normal
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
}

#[cfg(test)]
#[path = "../../tests/unit/app/preferences.rs"]
mod tests;
