mod browsing;
mod mutations;

use crate::db::{Project, Todo};
use ratatui::widgets::ListState;

enum ArchiveMode {
    Browsing,
    ConfirmClear(Vec<i64>),
}

pub(crate) struct ArchivedTasks {
    pub(crate) todos: Vec<Todo>,
    pub(crate) list: ListState,
    projects: Vec<Project>,
    mode: ArchiveMode,
}

#[cfg(test)]
#[path = "../../../tests/unit/app/archive.rs"]
mod tests;
