use super::*;
use crate::db::Priority;

fn siblings(database: &Database, parent: Option<i64>) -> Vec<i64> {
    let mut tasks: Vec<_> = database
        .list()
        .unwrap()
        .into_iter()
        .filter(|task| task.parent_id == parent)
        .collect();
    tasks.sort_by_key(|task| (task.position, task.id));
    tasks.into_iter().map(|task| task.id).collect()
}

#[test]
fn moves_whole_subtrees_without_changing_priority_or_completion() {
    let mut database = Database::memory();
    let first = database.add("First", None, Priority::High).unwrap();
    let second = database.add("Second", None, Priority::Low).unwrap();
    let third = database.add("Third", None, Priority::Mid).unwrap();
    let child = database
        .add("界 child", Some(second), Priority::High)
        .unwrap();
    database.toggle(child).unwrap();
    assert!(database.move_task(second, TaskMove::Up, 1).unwrap());
    assert_eq!(siblings(&database, None), [second, first, third]);
    assert!(
        database
            .move_task(second, TaskMove::Down, usize::MAX)
            .unwrap()
    );
    assert_eq!(siblings(&database, None), [first, third, second]);
    assert!(database.move_task(second, TaskMove::Indent, 1).unwrap());
    assert_eq!(siblings(&database, None), [first, third]);
    assert_eq!(siblings(&database, Some(third)), [second]);
    assert_eq!(siblings(&database, Some(second)), [child]);
    assert!(database.move_task(second, TaskMove::Outdent, 1).unwrap());
    assert_eq!(siblings(&database, None), [first, third, second]);
    let tasks = database.list().unwrap();
    let second = tasks.iter().find(|task| task.id == second).unwrap();
    assert_eq!(second.priority, Priority::Low);
    assert!(!second.done);
    assert!(tasks.iter().find(|task| task.id == child).unwrap().done);
    assert!(!database.move_task(first, TaskMove::Indent, 1).unwrap());
    assert!(!database.move_task(first, TaskMove::Outdent, 1).unwrap());
    assert!(!database.move_task(first, TaskMove::Up, 99).unwrap());
}

#[test]
fn nesting_counts_use_previous_siblings_and_place_outdented_tasks_after_parent() {
    let mut database = Database::memory();
    let parent = database.add("Parent", None, Priority::Mid).unwrap();
    let previous = database
        .add("Previous child", Some(parent), Priority::Mid)
        .unwrap();
    let task = database.add("Task", None, Priority::Mid).unwrap();
    let next = database.add("Next", None, Priority::Mid).unwrap();
    assert!(database.move_task(task, TaskMove::Indent, 2).unwrap());
    assert_eq!(siblings(&database, Some(previous)), [task]);
    assert!(database.move_task(task, TaskMove::Outdent, 99).unwrap());
    assert_eq!(siblings(&database, None), [parent, task, next]);
    assert_eq!(siblings(&database, Some(parent)), [previous]);
}

#[test]
fn ordering_rolls_back_parent_and_all_positions_on_failure() {
    let mut database = Database::memory();
    database.add("First", None, Priority::Mid).unwrap();
    let second = database.add("Second", None, Priority::High).unwrap();
    let before = database.list().unwrap();
    assert!(database.move_task(99999, TaskMove::Up, 1).is_err());
    database
        .connection
        .execute_batch(
            "CREATE TRIGGER reject_position BEFORE UPDATE OF position ON todos
         WHEN NEW.title = 'First' BEGIN SELECT RAISE(ABORT, 'test failure'); END;",
        )
        .unwrap();
    assert!(database.move_task(second, TaskMove::Indent, 1).is_err());
    assert_eq!(database.list().unwrap(), before);
    assert!(database.move_task(second, TaskMove::Up, 1).is_err());
    assert_eq!(database.list().unwrap(), before);
}

#[test]
fn manual_order_and_parent_links_survive_reopen_and_deletion_undo() {
    let folder = std::env::temp_dir().join(format!(
        "argv-todo-order-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path = folder.join("db.sql");
    let mut database = Database::open(&path).unwrap();
    let first = database.add("First", None, Priority::Low).unwrap();
    let second = database.add("Second", None, Priority::High).unwrap();
    let child = database.add("Child", Some(second), Priority::Mid).unwrap();
    database.move_task(second, TaskMove::Up, 1).unwrap();
    let snapshot = database.list().unwrap();
    let deleted = database
        .delete(
            &snapshot
                .iter()
                .filter(|task| task.id == second)
                .cloned()
                .collect::<Vec<_>>(),
        )
        .unwrap();
    database.restore(&deleted).unwrap();
    drop(database);
    let database = Database::open(&path).unwrap();
    assert_eq!(database.list().unwrap(), snapshot);
    assert_eq!(siblings(&database, None), [second, first]);
    assert_eq!(siblings(&database, Some(second)), [child]);
    drop(database);
    std::fs::remove_dir_all(folder).unwrap();
}
