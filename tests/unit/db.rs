use super::*;

impl Database {
    pub(crate) fn memory() -> Self {
        Self::initialize(Connection::open_in_memory().unwrap()).unwrap()
    }
}

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
    let folder =
        std::env::temp_dir().join(format!("argv-todo storage test-{}", std::process::id()));
    let path = folder.join("nested storage").join("db.sql");
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
