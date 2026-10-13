use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result, ensure};
use rusqlite::params;

use super::Database;

impl Database {
    pub(crate) fn restore_archived_tree(&mut self, id: i64) -> Result<Vec<i64>> {
        let transaction = self.connection.transaction()?;
        let archived: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM todos WHERE id = ?1 AND deleted = 1)",
            [id],
            |row| row.get(0),
        )?;
        ensure!(
            archived,
            "Archived task no longer exists; press Ctrl-r to refresh"
        );
        // Include ancestors so the restored tree never remains under a hidden parent.
        let restored = {
            let mut statement = transaction.prepare(
                "WITH RECURSIVE subtree(id) AS (
                     SELECT id FROM todos WHERE id = ?1
                     UNION
                     SELECT todos.id FROM todos JOIN subtree ON todos.parent_id = subtree.id
                 ), ancestors(id, parent_id) AS (
                     SELECT id, parent_id FROM todos WHERE id = ?1
                     UNION
                     SELECT todos.id, todos.parent_id FROM todos
                     JOIN ancestors ON todos.id = ancestors.parent_id
                 )
                 SELECT id FROM todos WHERE deleted = 1 AND
                 (id IN (SELECT id FROM subtree) OR id IN (SELECT id FROM ancestors))",
            )?;
            statement
                .query_map([id], |row| row.get::<_, i64>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        for id in &restored {
            let mismatched_project: bool = transaction.query_row(
                "SELECT EXISTS(SELECT 1 FROM todos child JOIN todos parent ON parent.id = child.parent_id
                 WHERE child.id = ?1 AND child.project_id IS NOT parent.project_id)",
                [id],
                |row| row.get(0),
            )?;
            ensure!(
                !mismatched_project,
                "Parent task moved to another project; could not restore archived tasks"
            );
            transaction.execute("UPDATE todos SET deleted = 0 WHERE id = ?1", [id])?;
        }
        transaction
            .commit()
            .context("Could not save restored tasks")?;
        Ok(restored)
    }

    /// Permanently remove only the archived records included in the confirmation.
    pub(crate) fn clear_archived(&mut self, ids: &[i64]) -> Result<()> {
        let transaction = self.connection.transaction()?;
        transaction.execute_batch("PRAGMA defer_foreign_keys = ON;")?;
        let tasks = {
            let mut statement = transaction.prepare(
                "SELECT id, title, done, created_at, updated_at, parent_id, priority, position, note, project_id, deleted
                 FROM todos ORDER BY position, id",
            )?;
            statement
                .query_map([], Self::todo_from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        let removed: HashSet<_> = ids.iter().copied().collect();
        let archived: HashSet<_> = tasks
            .iter()
            .filter(|task| task.deleted)
            .map(|task| task.id)
            .collect();
        ensure!(
            removed.is_subset(&archived),
            "Archive changed; press Ctrl-r to refresh before clearing it"
        );
        let parents: HashMap<_, _> = tasks.iter().map(|task| (task.id, task.parent_id)).collect();
        // Detach every survivor, including newly archived tasks, before SQLite cascades.
        for task in tasks.iter().filter(|task| !removed.contains(&task.id)) {
            let mut parent_id = task.parent_id;
            while let Some(parent) = parent_id.filter(|parent| removed.contains(parent)) {
                parent_id = parents.get(&parent).copied().flatten();
            }
            if parent_id != task.parent_id {
                transaction.execute(
                    "UPDATE todos SET parent_id = ?1,
                     position = (SELECT COALESCE(MAX(position), 0) + 1 FROM todos
                                 WHERE parent_id IS ?1 AND project_id IS ?2)
                     WHERE id = ?3",
                    params![parent_id, task.project_id, task.id],
                )?;
            }
        }
        for id in &removed {
            transaction.execute("DELETE FROM todos WHERE id = ?1 AND deleted = 1", [id])?;
        }
        transaction
            .commit()
            .context("Could not save archive cleanup")?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/db/archive.rs"]
mod tests;
