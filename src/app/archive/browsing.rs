use anyhow::{Context, Result};
use ratatui::widgets::ListState;

use super::{ArchiveMode, ArchivedTasks};
use crate::app::App;
use crate::{
    db::Todo,
    vim_motion::{Motion, VimAction},
};

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

    pub(super) fn select(&mut self, index: usize) {
        self.list
            .select(self.todos.len().checked_sub(1).map(|last| index.min(last)));
    }
}

impl App {
    pub(in crate::app) fn open_archive(&mut self) -> Result<()> {
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
            mode: ArchiveMode::Browsing,
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

    pub(in crate::app) fn apply_archive(&mut self, action: VimAction) -> Result<()> {
        if self.archive_clear_count().is_some() {
            match action {
                VimAction::Submit => self.confirm_archive_clear()?,
                VimAction::Cancel | VimAction::ShowDeleted => {
                    if let Some(archive) = &mut self.archive {
                        archive.mode = ArchiveMode::Browsing;
                    }
                    self.message("Archive cleanup cancelled.");
                }
                _ => {}
            }
            return Ok(());
        }
        match action {
            VimAction::RestoreArchived => self.restore_archived()?,
            VimAction::ClearArchived => self.begin_archive_clear()?,
            VimAction::ShowDeleted | VimAction::Cancel => {
                self.archive = None;
                self.vim.end_input();
                self.message("Returned to active tasks.");
            }
            VimAction::Refresh => {
                self.reload_archive()?;
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
            // Other task commands must never reach the active tree from the archive.
            _ => {}
        }
        Ok(())
    }
}
