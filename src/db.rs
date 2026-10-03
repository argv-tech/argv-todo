use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};
use std::{collections::BTreeMap, path::Path, time::Duration};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub done: bool,
    pub parent_id: Option<i64>,
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

    fn initialize(mut connection: Connection) -> Result<Self> {
        connection.busy_timeout(Duration::from_secs(3))?;
        connection.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS todos (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 title TEXT NOT NULL CHECK(length(trim(title)) > 0),
                 done INTEGER NOT NULL DEFAULT 0 CHECK(done IN (0, 1)),
                 parent_id INTEGER REFERENCES todos(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
                 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now')),
                 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now'))
             );",
        )?;
        let has_parent = connection
            .prepare("PRAGMA table_info(todos)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<rusqlite::Result<Vec<_>>>()?
            .iter()
            .any(|column| column == "parent_id");
        let transaction = connection.transaction()?;
        if !has_parent {
            transaction.execute_batch(
                "ALTER TABLE todos ADD COLUMN parent_id INTEGER
                 REFERENCES todos(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED;",
            )?;
        }
        transaction
            .execute_batch("CREATE INDEX IF NOT EXISTS todos_parent_id ON todos(parent_id);")?;
        transaction.commit()?;
        Ok(Self { connection })
    }

    pub fn list(&self) -> Result<Vec<Todo>> {
        let mut statement = self.connection.prepare(
            "SELECT id, title, done, created_at, updated_at, parent_id FROM todos ORDER BY id ASC",
        )?;
        Ok(statement
            .query_map([], Self::todo_from_row)?
            .collect::<rusqlite::Result<_>>()?)
    }

    fn todo_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Todo> {
        Ok(Todo {
            id: row.get(0)?,
            title: row.get(1)?,
            done: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
            parent_id: row.get(5)?,
        })
    }

    pub fn add(&self, title: &str, parent_id: Option<i64>) -> Result<i64> {
        self.connection.execute(
            "INSERT INTO todos (title, parent_id) VALUES (?1, ?2)",
            params![title, parent_id],
        )?;
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

    /// Give the selected task and every descendant the same completion state.
    pub fn toggle(&mut self, id: i64) -> Result<usize> {
        let transaction = self.connection.transaction()?;
        let done = transaction
            .query_row("SELECT done FROM todos WHERE id = ?1", [id], |row| {
                row.get::<_, bool>(0)
            })
            .optional()?
            .context("Task no longer exists; press Ctrl-r to refresh")?;
        let changed = transaction.execute(
            "WITH RECURSIVE subtree(id) AS (
                 SELECT id FROM todos WHERE id = ?1
                 UNION
                 SELECT todos.id FROM todos JOIN subtree ON todos.parent_id = subtree.id
             )
             UPDATE todos SET done = ?2, updated_at = strftime('%Y-%m-%d %H:%M:%S', 'now')
             WHERE id IN (SELECT id FROM subtree)",
            params![id, !done],
        )?;
        transaction.commit()?;
        Ok(changed)
    }

    /// Snapshot and delete whole subtrees atomically, including any newer children.
    pub fn delete(&mut self, todos: &[Todo]) -> Result<Vec<Todo>> {
        let transaction = self.connection.transaction()?;
        let mut deleted = BTreeMap::new();
        {
            let mut statement = transaction.prepare(
                "WITH RECURSIVE subtree(id) AS (
                     SELECT id FROM todos WHERE id = ?1
                     UNION
                     SELECT todos.id FROM todos JOIN subtree ON todos.parent_id = subtree.id
                 )
                 SELECT id, title, done, created_at, updated_at, parent_id
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

    pub fn restore(&mut self, todos: &[Todo]) -> Result<()> {
        let transaction = self.connection.transaction()?;
        for todo in todos {
            transaction.execute(
                "INSERT INTO todos (id, title, done, created_at, updated_at, parent_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![todo.id, todo.title, todo.done, todo.created_at, todo.updated_at, todo.parent_id]
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
        let id = db.add("Read Rust's book 📚", None).unwrap();
        db.rename(id, "Read chapter 2").unwrap();
        db.toggle(id).unwrap();
        let todos = db.list().unwrap();
        assert_eq!(todos[0].title, "Read chapter 2");
        assert!(todos[0].done);
        db.delete(&todos).unwrap();
        let newer = db.add("New task", None).unwrap();
        assert!(newer > id);
        db.restore(&todos).unwrap();
        assert_eq!(db.list().unwrap()[0], todos[0]);
        assert!(db.add("   ", None).is_err());
    }

    #[test]
    fn disk_storage_creates_parent_and_survives_reopen() {
        let folder = std::env::temp_dir().join(format!("todo-rs-test-{}", std::process::id()));
        let path = folder.join("nested/db.sql");
        {
            let db = Database::open(&path).unwrap();
            db.add("Persist this", None).unwrap();
        }
        let db = Database::open(&path).unwrap();
        assert_eq!(db.list().unwrap()[0].title, "Persist this");
        drop(db);
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn toggle_sets_the_entire_subtree_without_changing_ancestors_or_siblings() {
        let mut db = Database::memory();
        let parent = db.add("Parent", None).unwrap();
        let unrelated = db.add("Unrelated", None).unwrap();
        let child = db.add("Child", Some(parent)).unwrap();
        let grandchild = db.add("Grandchild", Some(child)).unwrap();
        let sibling = db.add("Sibling", Some(parent)).unwrap();
        db.toggle(grandchild).unwrap();
        assert_eq!(db.toggle(parent).unwrap(), 4);
        assert!(
            db.list()
                .unwrap()
                .iter()
                .all(|todo| todo.done == (todo.id != unrelated))
        );
        assert_eq!(db.toggle(parent).unwrap(), 4);
        assert!(db.list().unwrap().iter().all(|todo| !todo.done));
        assert_eq!(db.toggle(child).unwrap(), 2);
        assert!(
            db.list()
                .unwrap()
                .iter()
                .all(|todo| { todo.done == (todo.id == child || todo.id == grandchild) })
        );
        assert!(
            !db.list()
                .unwrap()
                .iter()
                .find(|todo| todo.id == sibling)
                .unwrap()
                .done
        );
        let expected = db.list().unwrap();
        assert!(db.toggle(9999).is_err());
        assert_eq!(db.list().unwrap(), expected);
        // A database failure must never leave a half-completed task tree.
        db.connection
            .execute_batch(
                "CREATE TRIGGER reject_completion BEFORE UPDATE OF done ON todos
             WHEN NEW.title = 'Sibling' AND NEW.done = 1
             BEGIN SELECT RAISE(ABORT, 'test failure'); END;",
            )
            .unwrap();
        assert!(db.toggle(parent).is_err());
        assert_eq!(db.list().unwrap(), expected);
    }

    #[test]
    fn migrates_existing_database_without_changing_tasks() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE todos (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 title TEXT NOT NULL,
                 done INTEGER NOT NULL DEFAULT 0,
                 created_at TEXT NOT NULL DEFAULT '2026-01-01 00:00:00',
                 updated_at TEXT NOT NULL DEFAULT '2026-01-02 00:00:00'
             );
             INSERT INTO todos (title, done) VALUES ('Existing task', 1);",
            )
            .unwrap();
        let db = Database::initialize(connection).unwrap();
        let existing = db.list().unwrap()[0].clone();
        assert_eq!(existing.title, "Existing task");
        assert!(existing.done);
        assert_eq!(existing.parent_id, None);
        assert_eq!(existing.created_at, "2026-01-01 00:00:00");
        let child = db.add("Child", Some(existing.id)).unwrap();
        assert_eq!(db.list().unwrap()[1].parent_id, Some(existing.id));
        assert!(db.add("Orphan", Some(child + 999)).is_err());
        // Reopening an upgraded schema is idempotent.
        let db = Database::initialize(db.connection).unwrap();
        assert_eq!(db.list().unwrap()[0], existing);
        assert_eq!(db.list().unwrap().len(), 2);
    }

    #[test]
    fn delete_snapshots_new_descendants_and_undo_restores_entire_tree() {
        let mut db = Database::memory();
        let parent = db.add("Parent", None).unwrap();
        db.add("Unrelated", None).unwrap();
        let stale_roots = db.list().unwrap();
        let child = db.add("Child", Some(parent)).unwrap();
        let grandchild = db.add("Grandchild", Some(child)).unwrap();
        db.add("Sibling", Some(parent)).unwrap();
        db.toggle(grandchild).unwrap();
        let expected = db.list().unwrap();
        let deleted = db.delete(&stale_roots[..1]).unwrap();
        assert_eq!(deleted.len(), 4);
        assert_eq!(db.list().unwrap(), expected[1..2]);
        db.restore(&deleted).unwrap();
        assert_eq!(db.list().unwrap(), expected);
    }
}
