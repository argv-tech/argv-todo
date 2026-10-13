use anyhow::Result;
use rusqlite::Connection;
use std::time::Duration;

use super::Database;

impl Database {
    pub(super) fn initialize(mut connection: Connection) -> Result<Self> {
        connection.busy_timeout(Duration::from_secs(3))?;
        connection.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS projects (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 name TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK(length(trim(name)) > 0)
             );
             CREATE TABLE IF NOT EXISTS todos (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 title TEXT NOT NULL CHECK(length(trim(title)) > 0),
                 note TEXT NOT NULL DEFAULT '',
                 project_id INTEGER REFERENCES projects(id),
                 done INTEGER NOT NULL DEFAULT 0 CHECK(done IN (0, 1)),
                 deleted INTEGER NOT NULL DEFAULT 0 CHECK(deleted IN (0, 1)),
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
        if !columns.iter().any(|column| column == "deleted") {
            transaction.execute_batch("ALTER TABLE todos ADD COLUMN deleted INTEGER NOT NULL DEFAULT 0 CHECK(deleted IN (0, 1));")?;
        }
        if !columns.iter().any(|column| column == "project_id") {
            transaction.execute_batch(
                "ALTER TABLE todos ADD COLUMN project_id INTEGER REFERENCES projects(id);",
            )?;
        }
        if !columns.iter().any(|column| column == "note") {
            transaction
                .execute_batch("ALTER TABLE todos ADD COLUMN note TEXT NOT NULL DEFAULT '';")?;
        }
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
        transaction.execute_batch(
            "CREATE INDEX IF NOT EXISTS todos_parent_id ON todos(parent_id);
             CREATE INDEX IF NOT EXISTS todos_project_id ON todos(project_id);",
        )?;
        transaction.commit()?;
        Ok(Self { connection })
    }
}
