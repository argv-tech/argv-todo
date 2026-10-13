use super::*;
use crate::db::Priority;

#[test]
fn conversion_preserves_tree_metadata_and_rolls_back_name_or_subtree_errors() {
    let mut db = Database::memory();
    let root = db.add("Reading 界é👩‍💻", None, Priority::High).unwrap();
    let child = db.add("Child", Some(root), Priority::Low).unwrap();
    db.add("Hidden", Some(child), Priority::Mid).unwrap();
    db.update_note(root, "Root note").unwrap();
    db.toggle(child).unwrap();
    let before = db.list().unwrap();
    assert!(db.convert_task_to_project(child).is_err());
    assert!(db.convert_task_to_project(999).is_err());
    assert_eq!(db.list().unwrap(), before);
    assert!(db.projects().unwrap().is_empty());
    db.connection.execute_batch("CREATE TRIGGER reject_conversion BEFORE UPDATE OF project_id ON todos WHEN NEW.title = 'Hidden' BEGIN SELECT RAISE(ABORT, 'rejected'); END;").unwrap();
    assert!(db.convert_task_to_project(root).is_err());
    assert!(db.projects().unwrap().is_empty());
    assert_eq!(db.list().unwrap(), before);
    db.connection
        .execute_batch("DROP TRIGGER reject_conversion;")
        .unwrap();
    let project = db.convert_task_to_project(root).unwrap();
    assert_eq!(db.projects().unwrap()[0].name, "Reading 界é👩‍💻");
    let after = db.list().unwrap();
    for old in &before {
        let new = after.iter().find(|task| task.id == old.id).unwrap();
        assert_eq!(new.project_id, Some(project));
        assert_eq!(
            (
                &new.title,
                &new.note,
                new.done,
                new.parent_id,
                new.priority,
                &new.created_at
            ),
            (
                &old.title,
                &old.note,
                old.done,
                old.parent_id,
                old.priority,
                &old.created_at
            )
        );
    }
    assert!(db.convert_task_to_project(root).is_err());
    assert_eq!(db.list().unwrap(), after);
    assert_eq!(db.projects().unwrap().len(), 1);
    let reserved = db.add("Inbox", None, Priority::Mid).unwrap();
    assert!(db.convert_task_to_project(reserved).is_err());
}
