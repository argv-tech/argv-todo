use super::*;
use crate::db::TaskMove;
use rusqlite::Connection;

#[test]
fn projects_inherit_on_creation_and_survive_moves_delete_restore_and_reopen() {
    let mut db = Database::memory();
    let project = db.create_project(" Work 界 ").unwrap();
    assert_eq!(db.projects().unwrap()[0].name, "Work 界");
    for invalid in ["", "  ", "work 界", "Inbox", "inbox"] {
        assert!(db.create_project(invalid).is_err());
    }
    let root = db
        .add_project_root("Root", Priority::High, Some(project))
        .unwrap();
    let child = db.add("Child", Some(root), Priority::Low).unwrap();
    let grandchild = db.add("Grandchild", Some(child), Priority::Mid).unwrap();
    let sibling = db
        .add_relative("Sibling", root, false, Priority::High)
        .unwrap();
    db.update_note(grandchild, "A saved note 👩‍💻").unwrap();
    db.toggle(grandchild).unwrap();
    assert!(
        db.list()
            .unwrap()
            .iter()
            .all(|task| task.project_id == Some(project))
    );
    let before = db.list().unwrap();
    db.assign_project(child, Some(project)).unwrap();
    assert_eq!(db.list().unwrap(), before);
    db.assign_project(child, None).unwrap();
    let moved = db.list().unwrap();
    for id in [child, grandchild] {
        assert_eq!(
            moved.iter().find(|task| task.id == id).unwrap().project_id,
            None
        );
    }
    assert_eq!(
        moved
            .iter()
            .find(|task| task.id == child)
            .unwrap()
            .parent_id,
        None
    );
    let nested = moved.iter().find(|task| task.id == grandchild).unwrap();
    assert_eq!(nested.parent_id, Some(child));
    assert!(nested.done);
    assert_eq!(nested.note, "A saved note 👩‍💻");
    assert_eq!(
        moved
            .iter()
            .find(|task| task.id == sibling)
            .unwrap()
            .project_id,
        Some(project)
    );
    let deleted = db.delete(&moved).unwrap();
    db.restore(&deleted).unwrap();
    assert_eq!(db.list().unwrap(), moved);
    let db = Database::initialize(db.connection).unwrap();
    assert_eq!(db.list().unwrap(), moved);
    assert_eq!(db.projects().unwrap()[0].id, project);
}

#[test]
fn assignment_is_atomic_and_ordering_cannot_cross_projects() {
    let mut db = Database::memory();
    let project = db.create_project("Work").unwrap();
    let root = db.add("Root", None, Priority::Mid).unwrap();
    db.add("Child", Some(root), Priority::Mid).unwrap();
    let foreign_root = db
        .add_project_root("Work root", Priority::Mid, Some(project))
        .unwrap();
    let before = db.list().unwrap();
    assert!(!db.move_task(foreign_root, TaskMove::Indent, 1).unwrap());
    assert_eq!(db.list().unwrap(), before);
    assert!(db.assign_project(root, Some(999)).is_err());
    assert!(db.assign_project(999, Some(project)).is_err());
    assert!(
        db.add_project_root("Invalid", Priority::Mid, Some(999))
            .is_err()
    );
    db.connection.execute_batch("CREATE TRIGGER reject_project BEFORE UPDATE OF project_id ON todos WHEN NEW.title = 'Child' BEGIN SELECT RAISE(ABORT, 'rejected'); END;").unwrap();
    assert!(db.assign_project(root, Some(project)).is_err());
    assert_eq!(db.list().unwrap(), before);
    db.connection
        .execute_batch("DROP TRIGGER reject_project;")
        .unwrap();
    db.assign_project(root, Some(project)).unwrap();
    let tasks = db.list().unwrap();
    assert!(tasks.iter().all(|task| task.project_id == Some(project)));
    assert!(
        tasks.iter().find(|task| task.id == root).unwrap().position
            > tasks
                .iter()
                .find(|task| task.id == foreign_root)
                .unwrap()
                .position
    );
}

#[test]
fn migration_keeps_existing_tree_notes_and_timestamps_in_inbox() {
    let connection = Connection::open_in_memory().unwrap();
    connection.execute_batch("CREATE TABLE todos (id INTEGER PRIMARY KEY, title TEXT NOT NULL, note TEXT NOT NULL, done INTEGER NOT NULL, parent_id INTEGER REFERENCES todos(id), priority INTEGER NOT NULL, position INTEGER NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
    INSERT INTO todos VALUES (1, 'Parent', 'Original note', 0, NULL, 1, 42, 'created', 'updated'), (2, 'Child', 'Nested', 1, 1, 5, 19, 'created', 'updated');").unwrap();
    let db = Database::initialize(connection).unwrap();
    let tasks = db.list().unwrap();
    assert!(db.projects().unwrap().is_empty());
    assert!(tasks.iter().all(|task| task.project_id.is_none()
        && task.created_at == "created"
        && task.updated_at == "updated"));
    assert_eq!(
        (tasks[0].note.as_str(), tasks[0].position, tasks[0].priority),
        ("Original note", 42, Priority::High)
    );
    assert_eq!(
        (
            tasks[1].parent_id,
            tasks[1].position,
            tasks[1].done,
            tasks[1].note.as_str()
        ),
        (Some(1), 19, true, "Nested")
    );
}
