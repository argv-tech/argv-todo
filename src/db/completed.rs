use std::collections::{HashMap, HashSet};

use anyhow::{Result, ensure};
use rusqlite::params;

use super::{Database, DeleteMode, Todo};

pub(crate) enum TaskScope {
    All,
    Project(Option<i64>),
}

pub(crate) struct CompletedDeletion {
    tasks: Vec<Todo>,
    parents: Vec<ParentChange>,
    mode: DeleteMode,
}

struct ParentChange {
    id: i64,
    parent_id: Option<i64>,
    position: i64,
    project_id: Option<i64>,
    repaired_parent_id: Option<i64>,
}

impl CompletedDeletion {
    pub(crate) fn contains_any(&self, ids: &HashSet<i64>) -> bool {
        self.tasks.iter().any(|task| ids.contains(&task.id))
    }

    pub(crate) fn len(&self) -> usize {
        self.tasks.len()
    }
    pub(crate) fn first_id(&self) -> Option<i64> {
        self.tasks.first().map(|task| task.id)
    }
}

impl Database {
    /// Clear completed records, including hidden matches, while retaining unfinished descendants.
    pub(crate) fn delete_completed(
        &mut self,
        scope: TaskScope,
        mode: DeleteMode,
    ) -> Result<CompletedDeletion> {
        let transaction = self.connection.transaction()?;
        // Older schemas may have immediate parent constraints. Both ends of a
        // completed parent/child link are removed within this transaction.
        transaction.execute_batch("PRAGMA defer_foreign_keys = ON;")?;
        let all_tasks = {
            let mut statement = transaction.prepare(
                "SELECT id, title, done, created_at, updated_at, parent_id, priority, position, note, project_id, deleted, deadline FROM todos ORDER BY position, id",
            )?;
            statement
                .query_map([], Self::todo_from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        let tasks: Vec<_> = all_tasks
            .iter()
            .filter(|task| {
                task.done
                    && !task.deleted
                    && match scope {
                        TaskScope::All => true,
                        TaskScope::Project(project_id) => task.project_id == project_id,
                    }
            })
            .cloned()
            .collect();
        let removed: HashSet<_> = tasks.iter().map(|task| task.id).collect();
        let parent_ids: HashMap<_, _> = all_tasks
            .iter()
            .map(|task| (task.id, task.parent_id))
            .collect();
        let mut parents = Vec::new();
        // Detach survivors before hard deletion so SQLite's cascade cannot delete them.
        // The same repair makes soft-deleted ancestors invisible in the active tree.
        for task in all_tasks.iter().filter(|task| !removed.contains(&task.id)) {
            let mut parent_id = task.parent_id;
            while let Some(parent) = parent_id.filter(|parent| removed.contains(parent)) {
                parent_id = parent_ids.get(&parent).copied().flatten();
            }
            if parent_id != task.parent_id {
                transaction.execute(
                    "UPDATE todos SET parent_id = ?1,
                     position = (SELECT COALESCE(MAX(position), 0) + 1 FROM todos
                                 WHERE parent_id IS ?1 AND project_id IS ?2)
                     WHERE id = ?3",
                    params![parent_id, task.project_id, task.id],
                )?;
                parents.push(ParentChange {
                    id: task.id,
                    parent_id: task.parent_id,
                    position: task.position,
                    project_id: task.project_id,
                    repaired_parent_id: parent_id,
                });
            }
        }
        for task in &tasks {
            match mode {
                DeleteMode::Soft => {
                    transaction.execute("UPDATE todos SET deleted = 1 WHERE id = ?1", [task.id])?
                }
                DeleteMode::Hard => {
                    transaction.execute("DELETE FROM todos WHERE id = ?1", [task.id])?
                }
            };
        }
        transaction.commit()?;
        Ok(CompletedDeletion {
            tasks,
            parents,
            mode,
        })
    }

    pub(crate) fn restore_completed(&mut self, deletion: &CompletedDeletion) -> Result<()> {
        let transaction = self.connection.transaction()?;
        match deletion.mode {
            DeleteMode::Hard => Self::insert_records(&transaction, &deletion.tasks)?,
            DeleteMode::Soft => {
                for task in &deletion.tasks {
                    let changed = transaction.execute(
                        "UPDATE todos SET deleted = 0 WHERE id = ?1 AND deleted = 1",
                        [task.id],
                    )?;
                    ensure!(
                        changed == 1,
                        "Archived task no longer exists or was already restored; press Ctrl-r to refresh"
                    );
                }
            }
        }
        for parent in &deletion.parents {
            let changed = transaction.execute(
                "UPDATE todos SET parent_id = ?1, position = ?2 WHERE id = ?3
                 AND project_id IS ?4 AND parent_id IS ?5",
                params![
                    parent.parent_id,
                    parent.position,
                    parent.id,
                    parent.project_id,
                    parent.repaired_parent_id
                ],
            )?;
            ensure!(
                changed == 1,
                "A surviving task was moved or deleted; could not restore its parent"
            );
        }
        transaction.commit()?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/db/completed.rs"]
mod tests;
