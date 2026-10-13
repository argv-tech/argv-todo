use anyhow::Result;
use std::collections::HashSet;

use super::App;
use crate::{
    config::TaskView,
    db::{CompletedDeletion, DeleteMode, TaskScope, Todo},
    vim_motion::VimAction,
};

pub(super) enum DeletionUndo {
    Trees(Vec<Todo>),
    Completed(CompletedDeletion),
}

pub(super) enum PendingDelete {
    Tasks(Vec<Todo>),
    Completed,
    Project(i64),
}

impl App {
    pub(super) fn discard_archive_undo(&mut self, ids: &[i64]) {
        let ids: HashSet<_> = ids.iter().copied().collect();
        // Stale snapshots must not resurrect a cleared archive or undo an explicit restore.
        self.undo.retain(|deletion| match deletion {
            DeletionUndo::Trees(tasks) => !tasks.iter().any(|task| ids.contains(&task.id)),
            DeletionUndo::Completed(deletion) => !deletion.contains_any(&ids),
        });
    }

    pub(crate) fn completed_delete(&self) -> DeleteMode {
        self.config
            .as_ref()
            .map_or(DeleteMode::Soft, |config| config.completed_delete)
    }

    pub(crate) fn confirming_delete(&self) -> bool {
        self.pending_delete.is_some()
    }

    pub(super) fn begin_task_delete(&mut self, count: usize) -> Result<()> {
        let visible = self.visible_indices();
        let start = self.active_list().selected().unwrap_or(0);
        let targets: Vec<_> = visible
            .into_iter()
            .skip(start)
            .take(count)
            .map(|i| self.todos[i].clone())
            .collect();
        if targets.is_empty() {
            return Ok(());
        }
        let len = targets.len();
        self.pending_delete = Some(PendingDelete::Tasks(targets));
        self.message(format!(
            "Delete {len} task(s)? Enter confirms · Esc cancels."
        ));
        Ok(())
    }

    pub(super) fn begin_completed_clear(&mut self) -> Result<()> {
        // Live tasks are exactly what the cleanup would remove, so the
        // count here matches the confirmed run below.
        let scoped = self.task_view() == TaskView::Projects;
        let active = self.active_project_id();
        let count = self
            .todos
            .iter()
            .filter(|task| task.done && (!scoped || task.project_id == active))
            .count();
        if count == 0 {
            self.message("No completed tasks to delete.");
            return Ok(());
        }
        self.pending_delete = Some(PendingDelete::Completed);
        self.message(format!(
            "Clear {count} completed task(s)? Enter confirms · Esc cancels."
        ));
        Ok(())
    }

    pub(super) fn apply_pending_delete(&mut self, action: VimAction) -> Result<()> {
        match action {
            VimAction::Submit => self.confirm_pending_delete()?,
            VimAction::Cancel => {
                self.pending_delete = None;
                self.message("Deletion cancelled.");
            }
            _ => {}
        }
        Ok(())
    }

    fn confirm_pending_delete(&mut self) -> Result<()> {
        let Some(pending) = self.pending_delete.take() else {
            return Ok(());
        };
        match pending {
            PendingDelete::Tasks(targets) => {
                let deleted = self.database.delete(&targets)?;
                if deleted.is_empty() {
                    self.reload(None)?;
                    self.message("Task no longer exists.");
                    return Ok(());
                }
                let len = deleted.len();
                self.undo.push(DeletionUndo::Trees(deleted));
                self.reload(None)?;
                self.message(format!("Deleted {len} task(s). Press u to undo."));
            }
            PendingDelete::Completed => self.clear_completed()?,
            PendingDelete::Project(id) => {
                // Selection is frozen while confirming, but refuse to delete
                // a different project if state ever drifts.
                if self.active_project_id() != Some(id) {
                    self.message("Deletion cancelled: active project changed.");
                    return Ok(());
                }
                self.delete_active_project()?;
            }
        }
        Ok(())
    }

    pub(super) fn clear_completed(&mut self) -> Result<()> {
        let selected = self
            .selected_todo()
            .filter(|task| !task.done)
            .map(|task| task.id);
        let scope = if self.task_view() == TaskView::Projects {
            TaskScope::Project(self.active_project_id())
        } else {
            TaskScope::All
        };
        let mode = self.completed_delete();
        let deleted = self.database.delete_completed(scope, mode)?;
        let count = deleted.len();
        if count == 0 {
            self.message("No completed tasks to delete.");
            return Ok(());
        }
        self.undo.push(DeletionUndo::Completed(deleted));
        self.reload(selected)?;
        let action = if mode == DeleteMode::Soft {
            "Archived"
        } else {
            "Deleted"
        };
        self.message(format!(
            "{action} {count} completed task(s). Press u to undo."
        ));
        Ok(())
    }

    pub(super) fn undo_deletion(&mut self) -> Result<()> {
        let Some(deletion) = self.undo.last() else {
            self.message("No deletion to undo in this session.");
            return Ok(());
        };
        let selected = match deletion {
            DeletionUndo::Trees(tasks) => {
                self.database.restore(tasks)?;
                tasks.first().map(|task| task.id)
            }
            DeletionUndo::Completed(deletion) => {
                self.database.restore_completed(deletion)?;
                deletion.first_id()
            }
        };
        self.undo.pop();
        self.query.clear();
        self.reload(selected)?;
        self.message("Deletion undone.");
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/app/deletion.rs"]
mod tests;
