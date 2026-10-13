use anyhow::{Context, Result};
use ratatui::widgets::ListState;

use super::App;
use crate::{
    db::{Project, Todo},
    vim_motion::{Motion, VimAction},
};

pub(crate) struct ArchivedTasks {
    pub(crate) todos: Vec<Todo>,
    pub(crate) list: ListState,
    projects: Vec<Project>,
}

impl ArchivedTasks {
    pub(crate) fn selected(&self) -> Option<&Todo> {
        self.list.selected().and_then(|index| self.todos.get(index))
    }

    pub(crate) fn project_name(&self, project_id: Option<i64>) -> &str {
        match project_id {
            None => "Inbox",
            Some(id) => self
                .projects
                .iter()
                .find(|project| project.id == id)
                .map_or("Unknown project", |project| project.name.as_str()),
        }
    }

    fn select(&mut self, index: usize) {
        self.list
            .select(self.todos.len().checked_sub(1).map(|last| index.min(last)));
    }
}

impl App {
    pub(super) fn open_archive(&mut self) -> Result<()> {
        // Keep the active tree, query, folds, project and viewport intact underneath.
        let todos = self
            .database
            .list_deleted()
            .context("Could not load soft-deleted tasks")?;
        let projects = self
            .database
            .projects()
            .context("Could not load archived task projects")?;
        let mut archive = ArchivedTasks {
            todos,
            projects,
            list: ListState::default(),
        };
        archive.select(0);
        self.archive = Some(archive);
        self.vim.end_input();
        self.message("Showing soft-deleted tasks from all projects. A or Esc returns.");
        Ok(())
    }

    pub(crate) fn note_todo(&self) -> Option<&Todo> {
        match &self.archive {
            Some(archive) => archive.selected(),
            None => self.selected_todo(),
        }
    }

    pub(super) fn apply_archive(&mut self, action: VimAction) -> Result<()> {
        match action {
            VimAction::ShowDeleted | VimAction::Cancel => {
                self.archive = None;
                self.vim.end_input();
                self.message("Returned to active tasks.");
            }
            VimAction::Refresh => {
                let id = self
                    .archive
                    .as_ref()
                    .and_then(ArchivedTasks::selected)
                    .map(|todo| todo.id);
                let todos = self
                    .database
                    .list_deleted()
                    .context("Could not reload soft-deleted tasks")?;
                let projects = self
                    .database
                    .projects()
                    .context("Could not reload archived task projects")?;
                if let Some(archive) = &mut self.archive {
                    let previous = archive.list.selected().unwrap_or(0);
                    let selected = id
                        .and_then(|id| todos.iter().position(|todo| todo.id == id))
                        .unwrap_or(previous);
                    archive.todos = todos;
                    archive.projects = projects;
                    archive.select(selected);
                }
                self.message("Reloaded soft-deleted tasks.");
            }
            VimAction::Help => {
                self.help = true;
                self.help_view = Default::default();
            }
            VimAction::Move(Motion::Up | Motion::Down, count) => {
                if let Some(archive) = &mut self.archive {
                    let selected = archive.list.selected().unwrap_or(0);
                    archive.select(if matches!(action, VimAction::Move(Motion::Down, _)) {
                        selected.saturating_add(count)
                    } else {
                        selected.saturating_sub(count)
                    });
                }
            }
            VimAction::FileStart | VimAction::FileEnd => {
                if let Some(archive) = &mut self.archive {
                    archive.select(if action == VimAction::FileStart {
                        0
                    } else {
                        usize::MAX
                    });
                }
            }
            // Archived records are read-only; task commands must never reach the active tree.
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/app/archive.rs"]
mod tests;
