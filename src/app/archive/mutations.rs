use anyhow::{Context, Result};

use super::{ArchiveMode, ArchivedTasks};
use crate::app::App;

impl App {
    pub(crate) fn archive_clear_count(&self) -> Option<usize> {
        match &self.archive.as_ref()?.mode {
            ArchiveMode::Browsing => None,
            ArchiveMode::ConfirmClear(ids) => Some(ids.len()),
        }
    }

    pub(super) fn reload_archive(&mut self) -> Result<()> {
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
        Ok(())
    }

    pub(super) fn restore_archived(&mut self) -> Result<()> {
        let Some(id) = self
            .archive
            .as_ref()
            .and_then(ArchivedTasks::selected)
            .map(|todo| todo.id)
        else {
            self.message("No archived task to restore.");
            return Ok(());
        };
        let selected = self.selected_todo().map(|todo| todo.id);
        let restored = self
            .database
            .restore_archived_tree(id)
            .context("Could not restore archived tasks")?;
        self.discard_archive_undo(&restored);
        self.reload(selected)?;
        self.reload_archive()?;
        self.message(format!("Restored {} archived task(s).", restored.len()));
        Ok(())
    }

    pub(super) fn begin_archive_clear(&mut self) -> Result<()> {
        self.reload_archive()?;
        let Some(archive) = &mut self.archive else {
            return Ok(());
        };
        if archive.todos.is_empty() {
            self.message("No soft-deleted tasks to clear.");
            return Ok(());
        }
        archive.mode =
            ArchiveMode::ConfirmClear(archive.todos.iter().map(|todo| todo.id).collect());
        self.message("");
        Ok(())
    }

    pub(super) fn confirm_archive_clear(&mut self) -> Result<()> {
        let Some(archive) = &mut self.archive else {
            return Ok(());
        };
        let ArchiveMode::ConfirmClear(ids) =
            std::mem::replace(&mut archive.mode, ArchiveMode::Browsing)
        else {
            return Ok(());
        };
        let selected = self.selected_todo().map(|todo| todo.id);
        self.database
            .clear_archived(&ids)
            .context("Could not clear soft-deleted tasks")?;
        self.discard_archive_undo(&ids);
        self.reload(selected)?;
        self.reload_archive()?;
        self.message(format!(
            "Permanently cleared {} archived task(s).",
            ids.len()
        ));
        Ok(())
    }
}
