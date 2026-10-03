use anyhow::{Context, Result};
use rusqlite::{
    Connection, OptionalExtension, ToSql, params,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, Value, ValueRef},
};
use std::{collections::BTreeMap, path::Path, time::Duration};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    High = 1,
    #[default]
    Mid = 3,
    Low = 5,
}

impl Priority {
    pub fn label(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Mid => "mid",
            Self::Low => "low",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Low => Self::Mid,
            Self::Mid => Self::High,
            Self::High => Self::Low,
        }
    }
}

impl ToSql for Priority {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Integer(*self as i64)))
    }
}

impl FromSql for Priority {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            1 => Ok(Self::High),
            3 => Ok(Self::Mid),
            5 => Ok(Self::Low),
            value => Err(FromSqlError::OutOfRange(value)),
        }
    }
}

pub const DEFAULT_PRIORITY: Priority = Priority::Mid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub done: bool,
    pub parent_id: Option<i64>,
    pub priority: Priority,
    pub position: i64,
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
                 priority INTEGER NOT NULL DEFAULT 3 CHECK(priority IN (1, 3, 5)),
                 position INTEGER NOT NULL DEFAULT 0,
                 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now')),
                 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now'))
             );",
        )?;
        let columns = connection
            .prepare("PRAGMA table_info(todos)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let transaction = connection.transaction()?;
        if !columns.iter().any(|column| column == "parent_id") {
            transaction.execute_batch(
                "ALTER TABLE todos ADD COLUMN parent_id INTEGER
                 REFERENCES todos(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED;",
            )?;
        }
        if !columns.iter().any(|column| column == "priority") {
            transaction.execute_batch(
                "ALTER TABLE todos ADD COLUMN priority INTEGER NOT NULL DEFAULT 3
                 CHECK(priority IN (1, 3, 5));",
            )?;
        }
        if !columns.iter().any(|column| column == "position") {
            transaction.execute_batch(
                "ALTER TABLE todos ADD COLUMN position INTEGER NOT NULL DEFAULT 0;
                 UPDATE todos SET position = id;",
            )?;
        }
        // Normalize databases created with the earlier five-level priority scale.
        transaction.execute_batch(
            "UPDATE todos SET priority = CASE priority WHEN 2 THEN 1 WHEN 4 THEN 5 END
             WHERE priority IN (2, 4);",
        )?;
        transaction
            .execute_batch("CREATE INDEX IF NOT EXISTS todos_parent_id ON todos(parent_id);")?;
        transaction.commit()?;
        Ok(Self { connection })
    }

    pub fn list(&self) -> Result<Vec<Todo>> {
        let mut statement = self.connection.prepare(
            "SELECT id, title, done, created_at, updated_at, parent_id, priority, position
             FROM todos ORDER BY priority ASC, position ASC, id ASC",
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
            priority: row.get(6)?,
            position: row.get(7)?,
        })
    }

    pub fn add(&self, title: &str, parent_id: Option<i64>, priority: Priority) -> Result<i64> {
        self.connection.execute(
            "INSERT INTO todos (title, parent_id, priority, position)
             SELECT ?1, ?2, ?3, COALESCE(MAX(position), 0) + 1
             FROM todos WHERE parent_id IS ?2",
            params![title, parent_id, priority],
        )?;
        Ok(self.connection.last_insert_rowid())
    }

    pub fn add_relative(
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

    pub fn update(&self, id: i64, title: &str, priority: Priority) -> Result<()> {
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

    pub fn set_priority(&self, id: i64, priority: Priority) -> Result<()> {
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
                 SELECT id, title, done, created_at, updated_at, parent_id, priority, position
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
                "INSERT INTO todos (id, title, done, created_at, updated_at, parent_id, priority, position) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![todo.id, todo.title, todo.done, todo.created_at, todo.updated_at, todo.parent_id, todo.priority, todo.position]
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
        let id = db.add("Read Rust's book 📚", None, Priority::Mid).unwrap();
        db.update(id, "Read chapter 2", Priority::High).unwrap();
        db.toggle(id).unwrap();
        let todos = db.list().unwrap();
        assert_eq!(todos[0].title, "Read chapter 2");
        assert!(todos[0].done);
        assert_eq!(todos[0].priority, Priority::High);
        db.delete(&todos).unwrap();
        let newer = db.add("New task", None, Priority::Mid).unwrap();
        assert!(newer > id);
        db.restore(&todos).unwrap();
        assert_eq!(db.list().unwrap()[0], todos[0]);
        assert!(db.add("   ", None, Priority::Mid).is_err());
    }

    #[test]
    fn disk_storage_creates_parent_and_survives_reopen() {
        let folder = std::env::temp_dir().join(format!("todo-rs-test-{}", std::process::id()));
        let path = folder.join("nested/db.sql");
        {
            let mut db = Database::open(&path).unwrap();
            let root = db.add("Persist this", None, Priority::High).unwrap();
            db.add_relative("Above", root, true, Priority::High)
                .unwrap();
            db.add_relative("Below", root, false, Priority::High)
                .unwrap();
        }
        let db = Database::open(&path).unwrap();
        let todos = db.list().unwrap();
        assert_eq!(
            todos
                .iter()
                .map(|todo| todo.title.as_str())
                .collect::<Vec<_>>(),
            ["Above", "Persist this", "Below"]
        );
        assert!(todos.iter().all(|todo| todo.priority == Priority::High));
        drop(db);
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn toggle_sets_the_entire_subtree_without_changing_ancestors_or_siblings() {
        let mut db = Database::memory();
        let parent = db.add("Parent", None, Priority::Mid).unwrap();
        let unrelated = db.add("Unrelated", None, Priority::Mid).unwrap();
        let child = db.add("Child", Some(parent), Priority::Mid).unwrap();
        let grandchild = db.add("Grandchild", Some(child), Priority::Mid).unwrap();
        let sibling = db.add("Sibling", Some(parent), Priority::Mid).unwrap();
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
        assert_eq!(existing.priority, Priority::Mid);
        assert_eq!(existing.position, existing.id);
        assert_eq!(existing.created_at, "2026-01-01 00:00:00");
        let child = db.add("Child", Some(existing.id), Priority::Mid).unwrap();
        assert_eq!(db.list().unwrap()[1].parent_id, Some(existing.id));
        assert!(db.add("Orphan", Some(child + 999), Priority::Mid).is_err());
        // Reopening an upgraded schema is idempotent.
        let db = Database::initialize(db.connection).unwrap();
        assert_eq!(db.list().unwrap()[0], existing);
        assert_eq!(db.list().unwrap().len(), 2);
    }

    #[test]
    fn sibling_insertion_rolls_back_order_on_failure() {
        let mut db = Database::memory();
        let root = db.add("Root", None, Priority::Mid).unwrap();
        let child = db.add("Child", Some(root), Priority::Mid).unwrap();
        db.add("Sibling", Some(root), Priority::Mid).unwrap();
        let expected = db.list().unwrap();
        assert!(db.add_relative(" ", child, true, Priority::Mid).is_err());
        assert_eq!(db.list().unwrap(), expected);
        assert!(
            db.add_relative("Orphan", 9999, false, Priority::Mid)
                .is_err()
        );
        assert_eq!(db.list().unwrap(), expected);
        db.add_relative("Before child", child, true, Priority::Mid)
            .unwrap();
        let todos = db.list().unwrap();
        assert_eq!(
            todos.iter().find(|todo| todo.id == root).unwrap(),
            &expected[0]
        );
        assert_eq!(
            todos
                .iter()
                .filter(|todo| todo.parent_id == Some(root))
                .map(|todo| todo.title.as_str())
                .collect::<Vec<_>>(),
            ["Before child", "Child", "Sibling"]
        );
    }

    #[test]
    fn delete_snapshots_new_descendants_and_undo_restores_entire_tree() {
        let mut db = Database::memory();
        let parent = db.add("Parent", None, Priority::Mid).unwrap();
        let unrelated = db.add("Unrelated", None, Priority::Mid).unwrap();
        let stale_roots = db.list().unwrap();
        let child = db.add("Child", Some(parent), Priority::Low).unwrap();
        let grandchild = db.add("Grandchild", Some(child), Priority::High).unwrap();
        db.add("Sibling", Some(parent), Priority::Low).unwrap();
        db.toggle(grandchild).unwrap();
        let expected = db.list().unwrap();
        let deleted = db.delete(&stale_roots[..1]).unwrap();
        assert_eq!(deleted.len(), 4);
        assert_eq!(
            db.list().unwrap(),
            expected
                .iter()
                .filter(|todo| todo.id == unrelated)
                .cloned()
                .collect::<Vec<_>>()
        );
        db.restore(&deleted).unwrap();
        assert_eq!(db.list().unwrap(), expected);
    }

    #[test]
    fn migrates_numeric_priorities_without_losing_hierarchy_or_dates() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE todos (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 title TEXT NOT NULL,
                 done INTEGER NOT NULL DEFAULT 0,
                 parent_id INTEGER REFERENCES todos(id),
                 priority INTEGER NOT NULL DEFAULT 3 CHECK(priority BETWEEN 1 AND 5),
                 created_at TEXT NOT NULL DEFAULT '2026-01-01 00:00:00',
                 updated_at TEXT NOT NULL DEFAULT '2026-01-02 00:00:00'
             );
             INSERT INTO todos (id, title, done, parent_id, priority) VALUES
                 (1, 'Parent', 0, NULL, 4),
                 (2, 'Child', 1, 1, 2),
                 (3, 'Root', 0, NULL, 3);",
            )
            .unwrap();
        let db = Database::initialize(connection).unwrap();
        let todos = db.list().unwrap();
        assert_eq!(
            todos.iter().map(|todo| todo.priority).collect::<Vec<_>>(),
            [Priority::High, Priority::Mid, Priority::Low]
        );
        assert_eq!(todos[0].parent_id, Some(1));
        assert!(todos[0].done);
        assert!(
            todos
                .iter()
                .all(|todo| todo.created_at == "2026-01-01 00:00:00"
                    && todo.updated_at == "2026-01-02 00:00:00")
        );
        let db = Database::initialize(db.connection).unwrap();
        assert_eq!(db.list().unwrap(), todos);
        let fresh = Database::memory();
        let id = fresh
            .add("Invalid priority check", None, Priority::Mid)
            .unwrap();
        assert!(
            fresh
                .connection
                .execute("UPDATE todos SET priority = 2 WHERE id = ?1", [id])
                .is_err()
        );
    }
}
