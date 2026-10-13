mod conversion;
mod editing;
mod tabs;

use ratatui::widgets::ListState;

use crate::db::Project;

pub(super) struct ProjectState {
    // Inbox always occupies index zero, so there is always an active tab.
    tabs: Vec<ProjectTab>,
    active: usize,
    picker: Option<ProjectPicker>,
}

struct ProjectTab {
    project: Option<Project>,
    list: ListState,
}

struct ProjectPicker {
    task_id: i64,
    list: ListState,
}

#[cfg(test)]
#[path = "../../../tests/unit/app/projects.rs"]
mod tests;
