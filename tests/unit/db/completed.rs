use super::*;
use crate::db::Priority;

fn count(db: &Database, archived: bool) -> usize {
    db.connection
        .query_row(
            "SELECT COUNT(*) FROM todos WHERE deleted = ?1",
            [archived],
            |row| row.get::<_, i64>(0),
        )
        .unwrap() as usize
}

#[test]
fn cleanup_preserves_unfinished_descendants_notes_projects_and_restores_exact_tree() {
    for mode in [DeleteMode::Soft, DeleteMode::Hard] {
        let mut db = Database::memory();
        let project = db.create_project("Work").unwrap();
        let root = db
            .add_project_root("Unfinished root", Priority::High, Some(project))
            .unwrap();
        let completed = db
            .add("Completed child", Some(root), Priority::Low)
            .unwrap();
        let nested = db
            .add("Completed grandchild", Some(completed), Priority::Mid)
            .unwrap();
        let survivor = db
            .add("Unfinished descendant", Some(nested), Priority::Low)
            .unwrap();
        db.update_note(survivor, "Keep me 界é👩‍💻").unwrap();
        db.toggle(completed).unwrap();
        db.toggle(survivor).unwrap();
        let external = db.add("Inbox completed", None, Priority::Mid).unwrap();
        db.toggle(external).unwrap();
        let before = db.list().unwrap();
        let deleted = db
            .delete_completed(TaskScope::Project(Some(project)), mode)
            .unwrap();
        assert_eq!(deleted.len(), 2);
        let remaining = db.list().unwrap();
        assert_eq!(remaining.len(), 3);
        let retained = remaining.iter().find(|task| task.id == survivor).unwrap();
        assert_eq!(retained.parent_id, Some(root));
        assert_eq!(retained.note, "Keep me 界é👩‍💻");
        assert_eq!(retained.project_id, Some(project));
        assert_eq!(
            count(&db, true),
            if mode == DeleteMode::Soft { 2 } else { 0 }
        );
        assert_eq!(count(&db, false), 3);
        db.restore_completed(&deleted).unwrap();
        assert_eq!(db.list().unwrap(), before);
        let deleted = db.delete_completed(TaskScope::All, mode).unwrap();
        assert_eq!(deleted.len(), 3);
        db.restore_completed(&deleted).unwrap();
        assert_eq!(db.list().unwrap(), before);
    }
}

#[test]
fn cleanup_and_undo_roll_back_all_changes_on_database_errors() {
    for mode in [DeleteMode::Soft, DeleteMode::Hard] {
        let mut db = Database::memory();
        let root = db.add("Root", None, Priority::Mid).unwrap();
        let child = db.add("Survivor", Some(root), Priority::Mid).unwrap();
        db.toggle(root).unwrap();
        db.toggle(child).unwrap();
        let before = db.list().unwrap();
        let event = if mode == DeleteMode::Soft {
            "UPDATE OF deleted"
        } else {
            "DELETE"
        };
        db.connection.execute_batch(&format!("CREATE TRIGGER reject_cleanup BEFORE {event} ON todos BEGIN SELECT RAISE(ABORT, 'rejected'); END;")).unwrap();
        assert!(db.delete_completed(TaskScope::All, mode).is_err());
        assert_eq!(db.list().unwrap(), before);
        assert_eq!(count(&db, true), 0);
        db.connection
            .execute_batch("DROP TRIGGER reject_cleanup;")
            .unwrap();
        let deletion = db.delete_completed(TaskScope::All, mode).unwrap();
        let cleaned = db.list().unwrap();
        db.connection.execute_batch("CREATE TRIGGER reject_restore BEFORE UPDATE OF parent_id ON todos BEGIN SELECT RAISE(ABORT, 'rejected'); END;").unwrap();
        assert!(db.restore_completed(&deletion).is_err());
        assert_eq!(db.list().unwrap(), cleaned);
        assert_eq!(count(&db, true), usize::from(mode == DeleteMode::Soft));
        db.connection
            .execute_batch("DROP TRIGGER reject_restore;")
            .unwrap();
        db.restore_completed(&deletion).unwrap();
        assert_eq!(db.list().unwrap(), before);
    }
}

#[test]
fn archived_tasks_remain_hidden_after_reopen_and_ignore_active_updates() {
    let mut db = Database::memory();
    let root = db.add("Root", None, Priority::Mid).unwrap();
    let archived = db.add("Completed", Some(root), Priority::Mid).unwrap();
    db.update_note(archived, "Archived note").unwrap();
    db.toggle(archived).unwrap();
    let deletion = db
        .delete_completed(TaskScope::All, DeleteMode::Soft)
        .unwrap();
    let mut db = Database::initialize(db.connection).unwrap();
    assert_eq!(db.list().unwrap().len(), 1);
    assert_eq!(count(&db, true), 1);
    assert!(db.toggle(archived).is_err());
    assert!(db.update_note(archived, "Overwrite").is_err());
    assert!(db.add("Child", Some(archived), Priority::Mid).is_err());
    assert!(
        db.add_relative("Sibling", archived, false, Priority::Mid)
            .is_err()
    );
    assert!(db.convert_task_to_project(archived).is_err());
    db.toggle(root).unwrap();
    let done: bool = db
        .connection
        .query_row("SELECT done FROM todos WHERE id = ?1", [archived], |row| {
            row.get(0)
        })
        .unwrap();
    assert!(done);
    db.restore_completed(&deletion).unwrap();
    assert_eq!(db.list().unwrap().len(), 2);
    assert_eq!(
        db.list()
            .unwrap()
            .iter()
            .find(|task| task.id == archived)
            .unwrap()
            .note,
        "Archived note"
    );
}

#[test]
fn hard_cleanup_retains_archived_descendants_and_undo_preserves_archive_state() {
    let mut db = Database::memory();
    let root = db.add("Root", None, Priority::Mid).unwrap();
    let child = db.add("Child", Some(root), Priority::Mid).unwrap();
    db.toggle(child).unwrap();
    db.delete_completed(TaskScope::All, DeleteMode::Soft)
        .unwrap();
    db.toggle(root).unwrap();
    let deletion = db
        .delete_completed(TaskScope::All, DeleteMode::Hard)
        .unwrap();
    assert!(db.list().unwrap().is_empty());
    assert_eq!(count(&db, true), 1);
    db.restore_completed(&deletion).unwrap();
    assert_eq!(db.list().unwrap().len(), 1);
    assert_eq!(count(&db, true), 1);
}

#[test]
fn hard_cleanup_and_restore_support_old_immediate_parent_constraints_in_either_order() {
    for positions in [(1, 2), (2, 1)] {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        connection.execute_batch("CREATE TABLE todos (id INTEGER PRIMARY KEY, title TEXT NOT NULL, note TEXT NOT NULL, done INTEGER NOT NULL, parent_id INTEGER REFERENCES todos(id), priority INTEGER NOT NULL, position INTEGER NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);").unwrap();
        connection.execute("INSERT INTO todos VALUES (1, 'Parent', 'note', 1, NULL, 1, ?1, 'created', 'updated'), (2, 'Child', 'child note', 1, 1, 5, ?2, 'created', 'updated')", params![positions.0, positions.1]).unwrap();
        let mut db = Database::initialize(connection).unwrap();
        let before = db.list().unwrap();
        let deletion = db
            .delete_completed(TaskScope::All, DeleteMode::Hard)
            .unwrap();
        assert!(db.list().unwrap().is_empty());
        db.restore_completed(&deletion).unwrap();
        assert_eq!(db.list().unwrap(), before);
    }
}

#[test]
fn undo_does_not_join_tasks_across_projects_after_later_moves() {
    for mode in [DeleteMode::Soft, DeleteMode::Hard] {
        let mut db = Database::memory();
        let root = db.add("Completed root", None, Priority::Mid).unwrap();
        let child = db
            .add("Unfinished child", Some(root), Priority::Mid)
            .unwrap();
        db.toggle(root).unwrap();
        db.toggle(child).unwrap();
        let deletion = db.delete_completed(TaskScope::All, mode).unwrap();
        let project = db.create_project("Moved").unwrap();
        db.assign_project(child, Some(project)).unwrap();
        let before = db.list().unwrap();
        assert!(db.restore_completed(&deletion).is_err());
        assert_eq!(db.list().unwrap(), before);
    }
    let mut db = Database::memory();
    let root = db.add("Root", None, Priority::Mid).unwrap();
    let child = db
        .add("Completed child", Some(root), Priority::Mid)
        .unwrap();
    db.toggle(child).unwrap();
    let deletion = db
        .delete_completed(TaskScope::All, DeleteMode::Hard)
        .unwrap();
    let project = db.create_project("Moved").unwrap();
    db.assign_project(root, Some(project)).unwrap();
    let before = db.list().unwrap();
    assert!(db.restore_completed(&deletion).is_err());
    assert_eq!(db.list().unwrap(), before);
}
