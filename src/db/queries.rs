use anyhow::{Context, Result};
use rusqlite::{OptionalExtension, params};

use super::{Database, Priority, Todo};

impl Database {
    pub(crate) fn list(&self) -> Result<Vec<Todo>> {
        let mut statement = self.connection.prepare(
            "SELECT id, title, done, created_at, updated_at, parent_id, priority, position
             FROM todos ORDER BY priority ASC, position ASC, id ASC",
        )?;
        Ok(statement
            .query_map([], Self::todo_from_row)?
            .collect::<rusqlite::Result<_>>()?)
    }

    pub(super) fn todo_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Todo> {
        Ok(Todo {
            id: row.get(0)?,
            title: row.get(1)?,
            done: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
            parent_id: row.get(5)?,
            priority: row.get(6)?,
            position: row.get(7)?,
        })
    }

    pub(crate) fn add(
        &self,
        title: &str,
        parent_id: Option<i64>,
        priority: Priority,
    ) -> Result<i64> {
        self.connection.execute(
            "INSERT INTO todos (title, parent_id, priority, position)
             SELECT ?1, ?2, ?3, COALESCE(MAX(position), 0) + 1
             FROM todos WHERE parent_id IS ?2",
            params![title, parent_id, priority],
        )?;
        Ok(self.connection.last_insert_rowid())
    }

    pub(crate) fn add_relative(
        &mut self,
        title: &str,
        id: i64,
        above: bool,
        priority: Priority,
    ) -> Result<i64> {
        let transaction = self.connection.transaction()?;
        let (parent_id, position) = transaction
            .query_row(
                "SELECT parent_id, position FROM todos WHERE id = ?1",
                [id],
                |row| Ok((row.get::<_, Option<i64>>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()?
            .context("Task no longer exists; press Ctrl-r to refresh")?;
        let position = position + i64::from(!above);
        transaction.execute(
            "UPDATE todos SET position = position + 1
             WHERE parent_id IS ?1 AND position >= ?2",
            params![parent_id, position],
        )?;
        transaction.execute(
            "INSERT INTO todos (title, parent_id, priority, position) VALUES (?1, ?2, ?3, ?4)",
            params![title, parent_id, priority, position],
        )?;
        let id = transaction.last_insert_rowid();
        transaction.commit()?;
        Ok(id)
    }

    pub(crate) fn update(&self, id: i64, title: &str, priority: Priority) -> Result<()> {
        let changed = self.connection.execute(
            "UPDATE todos SET title = ?1, priority = ?2,
             updated_at = strftime('%Y-%m-%d %H:%M:%S', 'now') WHERE id = ?3",
            params![title, priority, id],
        )?;
        anyhow::ensure!(
            changed == 1,
            "Task no longer exists; press Ctrl-r to refresh"
        );
        Ok(())
    }

    pub(crate) fn set_priority(&self, id: i64, priority: Priority) -> Result<()> {
        let changed = self.connection.execute(
            "UPDATE todos SET priority = ?1,
             updated_at = strftime('%Y-%m-%d %H:%M:%S', 'now') WHERE id = ?2",
            params![priority, id],
        )?;
        anyhow::ensure!(
            changed == 1,
            "Task no longer exists; press Ctrl-r to refresh"
        );
        Ok(())
    }
}
