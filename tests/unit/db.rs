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
    db.update_note(id, "Read examples too 界é👩‍💻").unwrap();
    db.update(id, "Read chapter 2", Priority::High).unwrap();
    db.toggle(id).unwrap();
    let todos = db.list().unwrap();
    assert_eq!(todos[0].title, "Read chapter 2");
    assert_eq!(todos[0].note, "Read examples too 界é👩‍💻");
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
        db.update_note(root, "Saved note with 'quotes' 界é👩‍💻")
            .unwrap();
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
    assert_eq!(todos[1].note, "Saved note with 'quotes' 界é👩‍💻");
    assert!(todos[0].note.is_empty() && todos[2].note.is_empty());
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
    assert!(existing.note.is_empty());
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
fn note_migration_preserves_current_hierarchy_order_and_timestamps() {
    let connection = Connection::open_in_memory().unwrap();
    connection
        .execute_batch(
            "CREATE TABLE todos (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             title TEXT NOT NULL,
             done INTEGER NOT NULL DEFAULT 0,
             parent_id INTEGER REFERENCES todos(id),
             priority INTEGER NOT NULL DEFAULT 3,
             position INTEGER NOT NULL DEFAULT 0,
             created_at TEXT NOT NULL DEFAULT '2026-01-01 00:00:00',
             updated_at TEXT NOT NULL DEFAULT '2026-01-02 00:00:00'
         );
         INSERT INTO todos (id, title, done, parent_id, priority, position) VALUES
             (1, 'Parent', 0, NULL, 1, 40),
             (2, 'Child', 1, 1, 5, 17);",
        )
        .unwrap();
    let db = Database::initialize(connection).unwrap();
    let todos = db.list().unwrap();
    assert!(todos.iter().all(|todo| todo.note.is_empty()
        && todo.created_at == "2026-01-01 00:00:00"
        && todo.updated_at == "2026-01-02 00:00:00"));
    assert_eq!(
        (todos[0].priority, todos[0].position, todos[0].done),
        (Priority::High, 40, false)
    );
    assert_eq!(
        (
            todos[1].priority,
            todos[1].position,
            todos[1].parent_id,
            todos[1].done
        ),
        (Priority::Low, 17, Some(1), true)
    );
    db.update_note(2, "Migrated note").unwrap();
    let saved = db.list().unwrap();
    let db = Database::initialize(db.connection).unwrap();
    assert_eq!(db.list().unwrap(), saved);
}

#[test]
fn note_updates_allow_empty_text_and_preserve_data_on_errors() {
    let db = Database::memory();
    let id = db.add("Task", None, Priority::Mid).unwrap();
    db.update_note(id, "A note with 'quotes' 界é👩‍💻").unwrap();
    let saved = db.list().unwrap();
    assert!(db.update_note(id + 999, "Missing task").is_err());
    assert_eq!(db.list().unwrap(), saved);
    db.connection
        .execute_batch(
            "CREATE TRIGGER reject_note BEFORE UPDATE OF note ON todos
         WHEN NEW.note = 'Rejected'
         BEGIN SELECT RAISE(ABORT, 'test failure'); END;",
        )
        .unwrap();
    assert!(db.update_note(id, "Rejected").is_err());
    assert_eq!(db.list().unwrap(), saved);
    db.update_note(id, "").unwrap();
    assert!(db.list().unwrap()[0].note.is_empty());
    assert_eq!(db.list().unwrap()[0].title, "Task");
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
    db.update_note(parent, "New note after stale snapshot")
        .unwrap();
    db.update_note(grandchild, "Nested note 界é👩‍💻").unwrap();
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

#[test]
fn deadlines_accept_only_real_datetimes() {
    for (text, expected) in [
        ("2026-10-20 14:30", Some((2026, 10, 20, 14, 30))),
        ("2026-01-05 00:00", Some((2026, 1, 5, 0, 0))),
        ("2024-02-29 23:59", Some((2024, 2, 29, 23, 59))),
        ("2000-02-29 12:00", Some((2000, 2, 29, 12, 0))),
        ("2026-10-20", Some((2026, 10, 20, 0, 0))),
        ("2026-02-29 12:00", None),
        ("1900-02-29 12:00", None),
        ("2026-10-20 24:00", None),
        ("2026-10-20 12:60", None),
        ("2026-10-20 1:30", None),
        ("2026-13-01 12:00", None),
        ("2026-04-31 12:00", None),
        ("2026-1-5 12:00", None),
        ("2026/10/20 12:00", None),
        ("2026-10-2012:00", None),
        ("abcd-ef-gh ij:kl", None),
        ("", None),
    ] {
        assert_eq!(split_deadline(text), expected, "{text:?}");
    }
    let (year, month, day, hour, minute) = split_deadline(&now()).unwrap();
    assert_eq!(
        format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}"),
        now()
    );
}

#[test]
fn deadlines_save_clear_and_survive_delete_undo() {
    let mut db = Database::memory();
    let id = db.add("Dated task", None, Priority::Mid).unwrap();
    assert_eq!(db.list().unwrap()[0].deadline, None);
    db.set_deadline(id, Some("2026-10-20 14:30")).unwrap();
    assert_eq!(
        db.list().unwrap()[0].deadline.as_deref(),
        Some("2026-10-20 14:30")
    );
    db.set_deadline(id, None).unwrap();
    assert_eq!(db.list().unwrap()[0].deadline, None);
    assert!(db.set_deadline(id + 100, Some("2026-10-20 14:30")).is_err());
    db.set_deadline(id, Some("2026-10-20 14:30")).unwrap();
    let todos = db.list().unwrap();
    let deleted = db.delete(&todos).unwrap();
    assert!(db.list().unwrap().is_empty());
    db.restore(&deleted).unwrap();
    assert_eq!(db.list().unwrap(), todos);
    // Urgency bands ignore completion state; missing and done deadlines stay muted.
    let dated = &db.list().unwrap()[0];
    for (deadline, done, expected) in [
        (Some("2026-10-20 11:59"), false, Urgency::Overdue),
        (Some("2026-10-20 12:00"), false, Urgency::Day),
        (Some("2026-10-21 11:59"), false, Urgency::Day),
        (Some("2026-10-21 12:00"), false, Urgency::Week),
        (Some("2026-10-27 11:59"), false, Urgency::Week),
        (Some("2026-10-27 12:00"), false, Urgency::Later),
        (Some("2999-12-31 23:59"), false, Urgency::Later),
        (None, false, Urgency::Later),
        (Some("2000-01-01 00:00"), true, Urgency::Later),
        (Some("not a deadline"), false, Urgency::Later),
    ] {
        assert_eq!(
            Todo {
                deadline: deadline.map(str::to_owned),
                done,
                ..dated.clone()
            }
            .urgency("2026-10-20 12:00"),
            expected,
            "{deadline:?}"
        );
    }
}

#[test]
fn migration_adds_nullable_deadline_to_legacy_databases() {
    let connection = Connection::open_in_memory().unwrap();
    connection
        .execute_batch(
            "CREATE TABLE projects (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 name TEXT NOT NULL UNIQUE COLLATE NOCASE
             );
             CREATE TABLE todos (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 title TEXT NOT NULL,
                 note TEXT NOT NULL DEFAULT '',
                 project_id INTEGER REFERENCES projects(id),
                 done INTEGER NOT NULL DEFAULT 0,
                 deleted INTEGER NOT NULL DEFAULT 0,
                 parent_id INTEGER REFERENCES todos(id),
                 priority INTEGER NOT NULL DEFAULT 3,
                 position INTEGER NOT NULL DEFAULT 0,
                 created_at TEXT NOT NULL DEFAULT '2026-01-01 00:00:00',
                 updated_at TEXT NOT NULL DEFAULT '2026-01-02 00:00:00'
             );
             INSERT INTO todos (id, title, note, done) VALUES (1, 'Legacy task', 'kept', 0);",
        )
        .unwrap();
    let db = Database::initialize(connection).unwrap();
    let todos = db.list().unwrap();
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].title, "Legacy task");
    assert_eq!(todos[0].note, "kept");
    assert_eq!(todos[0].deadline, None);
    db.set_deadline(1, Some("2026-10-20 14:30")).unwrap();
    assert_eq!(
        db.list().unwrap()[0].deadline.as_deref(),
        Some("2026-10-20 14:30")
    );
    let db = Database::initialize(db.connection).unwrap();
    assert_eq!(
        db.list().unwrap()[0].deadline.as_deref(),
        Some("2026-10-20 14:30")
    );
}

#[test]
fn migration_normalizes_bare_date_deadlines_to_midnight() {
    let db = Database::memory();
    let id = db.add("Bare date", None, Priority::Mid).unwrap();
    db.connection
        .execute(
            "UPDATE todos SET deadline = '2026-10-20' WHERE id = ?1",
            [id],
        )
        .unwrap();
    let db = Database::initialize(db.connection).unwrap();
    assert_eq!(
        db.list().unwrap()[0].deadline.as_deref(),
        Some("2026-10-20 00:00")
    );
    let db = Database::initialize(db.connection).unwrap();
    assert_eq!(
        db.list().unwrap()[0].deadline.as_deref(),
        Some("2026-10-20 00:00")
    );
}
