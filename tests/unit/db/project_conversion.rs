use super::*;
use crate::db::Priority;

#[test]
fn conversion_removes_root_preserves_descendants_and_rolls_back_errors() {
    let mut db = Database::memory();
    let root = db.add("Reading 界é👩‍💻", None, Priority::High).unwrap();
    let child = db.add("Child", Some(root), Priority::Low).unwrap();
    db.add("Hidden", Some(child), Priority::Mid).unwrap();
    let sibling = db.add("Sibling", Some(root), Priority::Low).unwrap();
    let archived = db.add("Archived", Some(root), Priority::Mid).unwrap();
    db.add("Archived child", Some(archived), Priority::Mid)
        .unwrap();
    db.update_note(child, "Child note").unwrap();
    db.update_note(archived, "Archived note").unwrap();
    db.toggle(child).unwrap();
    db.connection
        .execute(
            "UPDATE todos SET deleted = 1 WHERE id = ?1 OR parent_id = ?1",
            [archived],
        )
        .unwrap();
    let before = db.list().unwrap();
    let archived_before = db.list_deleted().unwrap();
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
    db.connection.execute_batch("CREATE TRIGGER reject_root_deletion BEFORE DELETE ON todos BEGIN SELECT RAISE(ABORT, 'rejected'); END;").unwrap();
    assert!(db.convert_task_to_project(root).is_err());
    assert!(db.projects().unwrap().is_empty());
    assert_eq!(db.list().unwrap(), before);
    assert_eq!(db.list_deleted().unwrap(), archived_before);
    db.connection
        .execute_batch("DROP TRIGGER reject_root_deletion;")
        .unwrap();
    let project = db.convert_task_to_project(root).unwrap();
    assert_eq!(db.projects().unwrap()[0].name, "Reading 界é👩‍💻");
    let after = db.list().unwrap();
    let archived_after = db.list_deleted().unwrap();
    assert_eq!(after.len(), before.len() - 1);
    assert_eq!(archived_after.len(), archived_before.len());
    assert!(
        after
            .iter()
            .chain(&archived_after)
            .all(|task| task.id != root)
    );
    for old in before
        .iter()
        .chain(&archived_before)
        .filter(|task| task.id != root)
    {
        let new = after
            .iter()
            .chain(&archived_after)
            .find(|task| task.id == old.id)
            .unwrap();
        let mut expected = old.clone();
        expected.project_id = Some(project);
        if expected.parent_id == Some(root) {
            expected.parent_id = None;
        }
        expected.updated_at.clone_from(&new.updated_at);
        assert_eq!(new, &expected);
    }
    assert!(
        after.iter().find(|task| task.id == child).unwrap().position
            < after
                .iter()
                .find(|task| task.id == sibling)
                .unwrap()
                .position
    );
    assert!(db.convert_task_to_project(root).is_err());
    assert_eq!(db.list().unwrap(), after);
    assert_eq!(db.projects().unwrap().len(), 1);
    let duplicate = db.add("Reading 界é👩‍💻", None, Priority::Mid).unwrap();
    let before_duplicate = db.list().unwrap();
    assert!(db.convert_task_to_project(duplicate).is_err());
    assert_eq!(db.list().unwrap(), before_duplicate);
    assert_eq!(db.projects().unwrap().len(), 1);
    let reserved = db.add("Inbox", None, Priority::Mid).unwrap();
    assert!(db.convert_task_to_project(reserved).is_err());
}

#[test]
fn conversion_without_children_creates_an_empty_project() {
    let mut db = Database::memory();
    let root = db.add("Empty project", None, Priority::Mid).unwrap();
    let project = db.convert_task_to_project(root).unwrap();
    assert_eq!(db.projects().unwrap()[0].id, project);
    assert_eq!(db.projects().unwrap()[0].name, "Empty project");
    assert!(db.list().unwrap().is_empty());
    assert!(db.list_deleted().unwrap().is_empty());
}
