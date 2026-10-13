use anyhow::{Context, Result, ensure};
use rusqlite::OptionalExtension;

use super::Database;

impl Database {
    pub(crate) fn convert_task_to_project(&mut self, id: i64) -> Result<i64> {
        let transaction = self.connection.transaction()?;
        let (title, parent_id) = transaction
            .query_row(
                "SELECT title, parent_id FROM todos WHERE id = ?1 AND deleted = 0",
                [id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<i64>>(1)?)),
            )
            .optional()?
            .context("Task no longer exists; press Ctrl-r to refresh")?;
        ensure!(
            parent_id.is_none(),
            "Only top-level tasks can become projects"
        );
        let project_id = Self::insert_project(&transaction, &title)?;
        Self::assign_project_tree(&transaction, id, Some(project_id))?;
        // Detach live and archived children before deleting the root to avoid cascading.
        transaction.execute(
            "UPDATE todos SET parent_id = NULL WHERE parent_id = ?1",
            [id],
        )?;
        transaction.execute("DELETE FROM todos WHERE id = ?1", [id])?;
        transaction.commit()?;
        Ok(project_id)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/db/project_conversion.rs"]
mod tests;
