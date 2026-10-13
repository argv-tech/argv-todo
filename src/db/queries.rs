use anyhow::{Context, Result};
use rusqlite::{OptionalExtension, params};

use super::{Database, Priority, Todo};

impl Database {
    pub(crate) fn list(&self) -> Result<Vec<Todo>> {
        self.list_by_deleted(false)
    }

    pub(crate) fn list_deleted(&self) -> Result<Vec<Todo>> {
        self.list_by_deleted(true)
    }

    fn list_by_deleted(&self, deleted: bool) -> Result<Vec<Todo>> {
        let mut statement = self.connection.prepare(
            "SELECT id, title, done, created_at, updated_at, parent_id, priority, position, note, project_id, deleted, deadline
             FROM todos WHERE deleted = ?1 ORDER BY priority ASC, position ASC, id ASC",
        )?;
        Ok(statement
            .query_map([deleted], Self::todo_from_row)?
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
            note: row.get(8)?,
            project_id: row.get(9)?,
            deleted: row.get(10)?,
            deadline: row.get(11)?,
        })
    }

    pub(crate) fn add(
        &self,
        title: &str,
        parent_id: Option<i64>,
        priority: Priority,
    ) -> Result<i64> {
        if let Some(parent_id) = parent_id {
            anyhow::ensure!(
                self.connection.query_row(
                    "SELECT EXISTS(SELECT 1 FROM todos WHERE id = ?1 AND deleted = 0)",
                    [parent_id],
                    |row| row.get::<_, bool>(0)
                )?,
                "Parent task no longer exists; press Ctrl-r to refresh"
            );
        }
        self.connection.execute(
            "INSERT INTO todos (title, parent_id, priority, position, project_id)
             SELECT ?1, ?2, ?3, COALESCE(MAX(position), 0) + 1,
                    (SELECT project_id FROM todos WHERE id = ?2 AND deleted = 0)
             FROM todos WHERE deleted = 0 AND parent_id IS ?2
             AND project_id IS (SELECT project_id FROM todos WHERE id = ?2 AND deleted = 0)",
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
        let (parent_id, position, project_id) = transaction
            .query_row(
                "SELECT parent_id, position, project_id FROM todos WHERE id = ?1 AND deleted = 0",
                [id],
                |row| {
                    Ok((
                        row.get::<_, Option<i64>>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Option<i64>>(2)?,
                    ))
                },
            )
            .optional()?
            .context("Task no longer exists; press Ctrl-r to refresh")?;
        let position = position + i64::from(!above);
        transaction.execute(
            "UPDATE todos SET position = position + 1
             WHERE parent_id IS ?1 AND position >= ?2 AND project_id IS ?3 AND deleted = 0",
            params![parent_id, position, project_id],
        )?;
        transaction.execute(
            "INSERT INTO todos (title, parent_id, priority, position, project_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![title, parent_id, priority, position, project_id],
        )?;
        let id = transaction.last_insert_rowid();
        transaction.commit()?;
        Ok(id)
    }

    pub(crate) fn update(&self, id: i64, title: &str, priority: Priority) -> Result<()> {
        let changed = self.connection.execute(
            "UPDATE todos SET title = ?1, priority = ?2,
             updated_at = strftime('%Y-%m-%d %H:%M:%S', 'now') WHERE id = ?3 AND deleted = 0",
            params![title, priority, id],
        )?;
        anyhow::ensure!(
            changed == 1,
            "Task no longer exists; press Ctrl-r to refresh"
        );
        Ok(())
    }

    pub(crate) fn update_note(&self, id: i64, note: &str) -> Result<()> {
        let changed = self
            .connection
            .execute(
                "UPDATE todos SET note = ?1,
             updated_at = strftime('%Y-%m-%d %H:%M:%S', 'now') WHERE id = ?2 AND deleted = 0",
                params![note, id],
            )
            .context("Could not save task note")?;
        anyhow::ensure!(
            changed == 1,
            "Task no longer exists; press Ctrl-r to refresh"
        );
        Ok(())
    }

    pub(crate) fn set_priority(&self, id: i64, priority: Priority) -> Result<()> {
        let changed = self.connection.execute(
            "UPDATE todos SET priority = ?1,
             updated_at = strftime('%Y-%m-%d %H:%M:%S', 'now') WHERE id = ?2 AND deleted = 0",
            params![priority, id],
        )?;
        anyhow::ensure!(
            changed == 1,
            "Task no longer exists; press Ctrl-r to refresh"
        );
        Ok(())
    }

    pub(crate) fn set_deadline(&self, id: i64, deadline: Option<&str>) -> Result<()> {
        let changed = self.connection.execute(
            "UPDATE todos SET deadline = ?1,
             updated_at = strftime('%Y-%m-%d %H:%M:%S', 'now') WHERE id = ?2 AND deleted = 0",
            params![deadline, id],
        )?;
        anyhow::ensure!(
            changed == 1,
            "Task no longer exists; press Ctrl-r to refresh"
        );
        Ok(())
    }
}
