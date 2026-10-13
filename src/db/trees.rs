use anyhow::{Context, Result};
use rusqlite::{OptionalExtension, params};
use std::collections::BTreeMap;

use super::{Database, Todo};

impl Database {
    /// Give the selected task and every descendant the same completion state.
    pub(crate) fn toggle(&mut self, id: i64) -> Result<usize> {
        let transaction = self.connection.transaction()?;
        let done = transaction
            .query_row(
                "SELECT done FROM todos WHERE id = ?1 AND deleted = 0",
                [id],
                |row| row.get::<_, bool>(0),
            )
            .optional()?
            .context("Task no longer exists; press Ctrl-r to refresh")?;
        let changed = transaction.execute(
            "WITH RECURSIVE subtree(id) AS (
                 SELECT id FROM todos WHERE id = ?1
                 UNION
                 SELECT todos.id FROM todos JOIN subtree ON todos.parent_id = subtree.id
             )
             UPDATE todos SET done = ?2, updated_at = strftime('%Y-%m-%d %H:%M:%S', 'now')
             WHERE id IN (SELECT id FROM subtree) AND deleted = 0",
            params![id, !done],
        )?;
        transaction.commit()?;
        Ok(changed)
    }

    /// Snapshot and delete whole subtrees atomically, including any newer children.
    pub(crate) fn delete(&mut self, todos: &[Todo]) -> Result<Vec<Todo>> {
        let transaction = self.connection.transaction()?;
        let mut deleted = BTreeMap::new();
        {
            let mut statement = transaction.prepare(
                "WITH RECURSIVE subtree(id) AS (
                     SELECT id FROM todos WHERE id = ?1
                     UNION
                     SELECT todos.id FROM todos JOIN subtree ON todos.parent_id = subtree.id
                 )
                 SELECT id, title, done, created_at, updated_at, parent_id, priority, position, note, project_id, deleted, deadline
                 FROM todos WHERE id IN (SELECT id FROM subtree) ORDER BY id",
            )?;
            for todo in todos {
                for row in statement.query_map([todo.id], Self::todo_from_row)? {
                    let row = row?;
                    deleted.insert(row.id, row);
                }
            }
        }
        for todo in todos {
            transaction.execute("DELETE FROM todos WHERE id = ?1", [todo.id])?;
        }
        transaction.commit()?;
        Ok(deleted.into_values().collect())
    }

    pub(crate) fn restore(&mut self, todos: &[Todo]) -> Result<()> {
        let transaction = self.connection.transaction()?;
        Self::insert_records(&transaction, todos)?;
        transaction.commit()?;
        Ok(())
    }

    pub(super) fn insert_records(
        transaction: &rusqlite::Transaction<'_>,
        todos: &[Todo],
    ) -> Result<()> {
        transaction.execute_batch("PRAGMA defer_foreign_keys = ON;")?;
        for todo in todos {
            transaction.execute(
                "INSERT INTO todos (id, title, done, created_at, updated_at, parent_id, priority, position, note, project_id, deleted, deadline) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![todo.id, todo.title, todo.done, todo.created_at, todo.updated_at, todo.parent_id, todo.priority, todo.position, todo.note, todo.project_id, todo.deleted, todo.deadline]
            )?;
        }
        for todo in todos {
            let mismatched_project = transaction.query_row(
                "SELECT EXISTS(SELECT 1 FROM todos child JOIN todos parent ON parent.id = child.parent_id
                 WHERE child.id = ?1 AND child.project_id IS NOT parent.project_id)",
                [todo.id], |row| row.get::<_, bool>(0),
            )?;
            anyhow::ensure!(
                !mismatched_project,
                "Parent task moved to another project; could not restore deleted tasks"
            );
        }
        Ok(())
    }
}
