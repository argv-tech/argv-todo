use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::{path::Path, time::Duration};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub done: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Could not create {}", parent.display()))?;
        }
        Self::initialize(Connection::open(path)?)
    }

    fn initialize(connection: Connection) -> Result<Self> {
        connection.busy_timeout(Duration::from_secs(3))?;
        connection.execute_batch(
            "PRAGMA journal_mode = WAL;
             CREATE TABLE IF NOT EXISTS todos (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 title TEXT NOT NULL CHECK(length(trim(title)) > 0),
                 done INTEGER NOT NULL DEFAULT 0 CHECK(done IN (0, 1)),
                 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now')),
                 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now'))
             );",
        )?;
        Ok(Self { connection })
    }

    pub fn list(&self) -> Result<Vec<Todo>> {
        let mut statement = self
            .connection
            .prepare("SELECT id, title, done, created_at, updated_at FROM todos ORDER BY id ASC")?;
        Ok(statement
            .query_map([], |row| {
                Ok(Todo {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    done: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?)
    }

    pub fn add(&self, title: &str) -> Result<i64> {
        self.connection
            .execute("INSERT INTO todos (title) VALUES (?1)", [title])?;
        Ok(self.connection.last_insert_rowid())
    }

    pub fn rename(&self, id: i64, title: &str) -> Result<()> {
        let changed = self.connection.execute(
            "UPDATE todos SET title = ?1, updated_at = strftime('%Y-%m-%d %H:%M:%S', 'now') WHERE id = ?2",
            params![title, id]
        )?;
        anyhow::ensure!(
            changed == 1,
            "Task no longer exists; press Ctrl-r to refresh"
        );
        Ok(())
    }

    pub fn toggle(&self, id: i64) -> Result<()> {
        let changed = self.connection.execute(
            "UPDATE todos SET done = 1 - done, updated_at = strftime('%Y-%m-%d %H:%M:%S', 'now') WHERE id = ?1", [id]
        )?;
        anyhow::ensure!(
            changed == 1,
            "Task no longer exists; press Ctrl-r to refresh"
        );
        Ok(())
    }

    pub fn delete(&mut self, todos: &[Todo]) -> Result<()> {
        let transaction = self.connection.transaction()?;
        for todo in todos {
            transaction.execute("DELETE FROM todos WHERE id = ?1", [todo.id])?;
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn restore(&mut self, todos: &[Todo]) -> Result<()> {
        let transaction = self.connection.transaction()?;
        for todo in todos {
            transaction.execute(
                "INSERT INTO todos (id, title, done, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![todo.id, todo.title, todo.done, todo.created_at, todo.updated_at]
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    #[cfg(test)]
    pub fn memory() -> Self {
        Self::initialize(Connection::open_in_memory().unwrap()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crud_and_undo_preserve_original_record() {
        let mut db = Database::memory();
        let id = db.add("Read Rust's book 📚").unwrap();
        db.rename(id, "Read chapter 2").unwrap();
        db.toggle(id).unwrap();
        let todos = db.list().unwrap();
        assert_eq!(todos[0].title, "Read chapter 2");
        assert!(todos[0].done);
        db.delete(&todos).unwrap();
        let newer = db.add("New task").unwrap();
        assert!(newer > id);
        db.restore(&todos).unwrap();
        assert_eq!(db.list().unwrap()[0], todos[0]);
        assert!(db.add("   ").is_err());
    }

    #[test]
    fn disk_storage_creates_parent_and_survives_reopen() {
        let folder = std::env::temp_dir().join(format!("todo-rs-test-{}", std::process::id()));
        let path = folder.join("nested/db.sql");
        {
            let db = Database::open(&path).unwrap();
            db.add("Persist this").unwrap();
        }
        let db = Database::open(&path).unwrap();
        assert_eq!(db.list().unwrap()[0].title, "Persist this");
        drop(db);
        std::fs::remove_dir_all(folder).unwrap();
    }
}
