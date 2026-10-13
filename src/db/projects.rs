use anyhow::{Context, Result, ensure};
use rusqlite::{OptionalExtension, params};

use super::{Database, Priority, Project};

impl Database {
    pub(crate) fn projects(&self) -> Result<Vec<Project>> {
        Ok(self
            .connection
            .prepare("SELECT id, name FROM projects ORDER BY id")?
            .query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?)
    }

    pub(crate) fn create_project(&self, name: &str) -> Result<i64> {
        Self::insert_project(&self.connection, name)
    }

    pub(super) fn insert_project(connection: &rusqlite::Connection, name: &str) -> Result<i64> {
        let name = name.trim();
        ensure!(!name.is_empty(), "Project name cannot be empty");
        ensure!(
            !name.eq_ignore_ascii_case("Inbox"),
            "Inbox is reserved for unassigned tasks"
        );
        connection
            .execute("INSERT INTO projects (name) VALUES (?1)", [name])
            .context("Could not create project; use a unique name")?;
        Ok(connection.last_insert_rowid())
    }

    pub(crate) fn add_project_root(
        &self,
        title: &str,
        priority: Priority,
        project_id: Option<i64>,
    ) -> Result<i64> {
        self.connection.execute(
            "INSERT INTO todos (title, priority, project_id, position)
             SELECT ?1, ?2, ?3, COALESCE(MAX(position), 0) + 1
             FROM todos WHERE parent_id IS NULL AND project_id IS ?3 AND deleted = 0",
            params![title, priority, project_id],
        )?;
        Ok(self.connection.last_insert_rowid())
    }

    /// Moving a child to another project makes it a root; its descendants keep their links.
    pub(crate) fn assign_project(&mut self, id: i64, project_id: Option<i64>) -> Result<()> {
        let transaction = self.connection.transaction()?;
        Self::assign_project_tree(&transaction, id, project_id)?;
        transaction.commit()?;
        Ok(())
    }

    pub(super) fn assign_project_tree(
        transaction: &rusqlite::Transaction<'_>,
        id: i64,
        project_id: Option<i64>,
    ) -> Result<()> {
        let current: Option<i64> = transaction
            .query_row(
                "SELECT project_id FROM todos WHERE id = ?1 AND deleted = 0",
                [id],
                |row| row.get(0),
            )
            .optional()?
            .context("Task no longer exists; press Ctrl-r to refresh")?;
        if let Some(project_id) = project_id {
            ensure!(
                transaction.query_row(
                    "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
                    [project_id],
                    |row| row.get::<_, bool>(0)
                )?,
                "Project no longer exists; press Ctrl-r to refresh"
            );
        }
        if current != project_id {
            transaction.execute(
                "UPDATE todos SET parent_id = NULL,
                 position = (SELECT COALESCE(MAX(position), 0) + 1 FROM todos
                             WHERE parent_id IS NULL AND project_id IS ?2)
                 WHERE id = ?1",
                params![id, project_id],
            )?;
            transaction.execute(
                "WITH RECURSIVE subtree(id) AS (
                     SELECT id FROM todos WHERE id = ?1
                     UNION SELECT todos.id FROM todos JOIN subtree ON todos.parent_id = subtree.id
                 )
                 UPDATE todos SET project_id = ?2, updated_at = strftime('%Y-%m-%d %H:%M:%S', 'now')
                 WHERE id IN (SELECT id FROM subtree)",
                params![id, project_id],
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/db/projects.rs"]
mod tests;
