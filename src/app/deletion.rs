use anyhow::Result;
use std::collections::HashSet;

use super::App;
use crate::{
    config::TaskView,
    db::{CompletedDeletion, DeleteMode, TaskScope, Todo},
};

pub(super) enum DeletionUndo {
    Trees(Vec<Todo>),
    Completed(CompletedDeletion),
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
