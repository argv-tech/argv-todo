use super::*;
use crate::db::{DeleteMode, Priority, TaskScope};

#[test]
fn restoring_a_tree_and_archived_parents_keeps_metadata_after_reopen() {
    let folder = std::env::temp_dir().join(format!("argv-restore-archive-{}", std::process::id()));
    let path = folder.join("db.sql");
    let mut db = Database::open(&path).unwrap();
    let project = db.create_project("Work").unwrap();
    let root = db
        .add_project_root("Root", Priority::High, Some(project))
        .unwrap();
    let child = db.add("Child 界é👩‍💻", Some(root), Priority::Low).unwrap();
    let grandchild = db.add("Grandchild", Some(child), Priority::Mid).unwrap();
    let sibling = db.add("Sibling", Some(root), Priority::Mid).unwrap();
    db.update_note(child, "Saved note 界é👩‍💻").unwrap();
    db.toggle(root).unwrap();
    let before = db.list().unwrap();
    db.delete_completed(TaskScope::All, DeleteMode::Soft)
        .unwrap();
    drop(db);
    let mut db = Database::open(&path).unwrap();
    let restored = db.restore_archived_tree(child).unwrap();
    assert_eq!(
        restored.iter().copied().collect::<HashSet<_>>(),
        HashSet::from([root, child, grandchild])
    );
    assert_eq!(db.list_deleted().unwrap()[0].id, sibling);
    assert_eq!(
        db.list().unwrap(),
        before
            .into_iter()
            .filter(|task| task.id != sibling)
            .collect::<Vec<_>>()
    );
    assert!(db.restore_archived_tree(child).is_err());
    drop(db);
    let db = Database::open(&path).unwrap();
    assert_eq!(db.list().unwrap().len(), 3);
    assert_eq!(db.list_deleted().unwrap().len(), 1);
    drop(db);
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn clearing_archive_preserves_live_and_newly_archived_descendants() {
    let mut db = Database::memory();
    let root = db.add("Live root", None, Priority::Mid).unwrap();
    let parent = db
        .add("Archived parent", Some(root), Priority::Mid)
        .unwrap();
    let child = db
        .add("Archived child", Some(parent), Priority::Mid)
        .unwrap();
    let survivor = db.add("Live survivor", Some(child), Priority::Low).unwrap();
    db.update_note(survivor, "Keep this note").unwrap();
    // Exercise a legacy/external database with live descendants under archived ancestors.
    db.connection
        .execute(
            "UPDATE todos SET deleted = 1 WHERE id IN (?1, ?2)",
            params![parent, child],
        )
        .unwrap();
    let ids = db
        .list_deleted()
        .unwrap()
        .iter()
        .map(|task| task.id)
        .collect::<Vec<_>>();
    let newer = db.add("New archive", None, Priority::Mid).unwrap();
    db.connection
        .execute(
            "UPDATE todos SET deleted = 1, parent_id = ?1 WHERE id = ?2",
            params![child, newer],
        )
        .unwrap();
    db.clear_archived(&ids).unwrap();
    assert_eq!(db.list_deleted().unwrap()[0].id, newer);
    assert_eq!(db.list_deleted().unwrap()[0].parent_id, Some(root));
    let live = db.list().unwrap();
    assert_eq!(live.len(), 2);
    let task = live.iter().find(|task| task.id == survivor).unwrap();
    assert_eq!(task.parent_id, Some(root));
    assert_eq!(task.note, "Keep this note");
    assert!(!task.done);
}

#[test]
fn archive_mutations_roll_back_and_reject_stale_confirmation() {
    let mut db = Database::memory();
    let root = db.add("Root", None, Priority::Mid).unwrap();
    let child = db.add("Child", Some(root), Priority::Mid).unwrap();
    db.toggle(root).unwrap();
    db.delete_completed(TaskScope::All, DeleteMode::Soft)
        .unwrap();
    let before = db.list_deleted().unwrap();
    db.connection.execute_batch(&format!("CREATE TRIGGER reject_restore BEFORE UPDATE OF deleted ON todos WHEN OLD.id = {child} BEGIN SELECT RAISE(ABORT, 'restore rejected'); END;")).unwrap();
    assert!(db.restore_archived_tree(root).is_err());
    assert_eq!(db.list_deleted().unwrap(), before);
    assert!(db.list().unwrap().is_empty());
    db.connection
        .execute_batch("DROP TRIGGER reject_restore;")
        .unwrap();
    let survivor = db.add("Survivor", None, Priority::Mid).unwrap();
    db.connection
        .execute(
            "UPDATE todos SET parent_id = ?1 WHERE id = ?2",
            params![child, survivor],
        )
        .unwrap();
    let live = db.list().unwrap();
    db.connection.execute_batch("CREATE TRIGGER reject_clear BEFORE DELETE ON todos BEGIN SELECT RAISE(ABORT, 'clear rejected'); END;").unwrap();
    assert!(db.clear_archived(&[root, child]).is_err());
    assert_eq!(db.list_deleted().unwrap(), before);
    assert_eq!(db.list().unwrap(), live);
    db.connection
        .execute_batch("DROP TRIGGER reject_clear;")
        .unwrap();
    db.restore_archived_tree(child).unwrap();
    assert!(db.clear_archived(&[root, child]).is_err());
    assert_eq!(db.list().unwrap().len(), 3);
    assert!(db.clear_archived(&[i64::MAX]).is_err());
    db.clear_archived(&[]).unwrap();
}

#[test]
fn restoring_archive_rejects_parents_that_moved_projects() {
    let mut db = Database::memory();
    let parent = db.add("Parent", None, Priority::Mid).unwrap();
    let child = db.add("Child", Some(parent), Priority::Mid).unwrap();
    db.toggle(child).unwrap();
    db.delete_completed(TaskScope::All, DeleteMode::Soft)
        .unwrap();
    let project = db.create_project("Moved").unwrap();
    db.connection
        .execute(
            "UPDATE todos SET project_id = ?1 WHERE id = ?2",
            params![project, parent],
        )
        .unwrap();
    let before = db.list_deleted().unwrap();
    assert!(db.restore_archived_tree(child).is_err());
    assert_eq!(db.list_deleted().unwrap(), before);
}
