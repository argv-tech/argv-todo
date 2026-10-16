use super::*;
use crate::config::{Config, TaskView};
use crate::vim_motion::{Motion, VimAction};

#[test]
fn deadline_sort_uses_branch_dates_and_priority_sort_ignores_them() {
    for view in [TaskView::Normal, TaskView::Split, TaskView::Projects] {
        let folder = std::env::temp_dir().join(format!(
            "argv-deadline-sort-{}-{view:?}",
            std::process::id()
        ));
        let database = crate::db::Database::memory();
        let undated = database.add("Undated root", None, Priority::High).unwrap();
        let later = database.add("Later root", None, Priority::Mid).unwrap();
        let earlier = database.add("Earlier root", None, Priority::Low).unwrap();
        let child = database.add("Child", Some(earlier), Priority::Low).unwrap();
        let grandchild = database
            .add("Grandchild", Some(child), Priority::Low)
            .unwrap();
        let sibling = database
            .add("Sibling", Some(earlier), Priority::High)
            .unwrap();
        database
            .set_deadline(later, Some("2030-01-01 12:00"))
            .unwrap();
        database
            .set_deadline(grandchild, Some("2030-01-01 09:00"))
            .unwrap();
        database
            .set_deadline(sibling, Some("2030-01-01 10:00"))
            .unwrap();
        let mut app = App::new(database).unwrap();
        let mut config = Config::load(&folder.join("tasks.sql"), false).unwrap();
        config.save_task_view(view).unwrap();
        config.save_sort_order(SortOrder::Deadline).unwrap();
        app.config = Some(config);
        app.reload(Some(grandchild)).unwrap();
        assert_eq!(
            ids(&app),
            [earlier, child, grandchild, sibling, later, undated]
        );
        assert_eq!(app.selected_todo().unwrap().id, grandchild);

        app.query = "root".into();
        assert_eq!(ids(&app), [earlier, later, undated]);
        app.query.clear();
        app.select_id(earlier);
        app.apply(VimAction::ToggleCollapse).unwrap();
        assert_eq!(ids(&app), [earlier, later, undated]);
        app.apply(VimAction::Move(Motion::Right, 1)).unwrap();
        assert_eq!(app.selected_todo().unwrap().id, child);

        app.database.set_deadline(grandchild, None).unwrap();
        app.reload(Some(grandchild)).unwrap();
        assert_eq!(
            ids(&app),
            [earlier, sibling, child, grandchild, later, undated]
        );
        app.database.set_deadline(sibling, None).unwrap();
        app.reload(Some(grandchild)).unwrap();
        assert_eq!(
            ids(&app),
            [later, undated, earlier, child, grandchild, sibling]
        );

        // Equal deadlines keep saved order, with no priority tie-breaker.
        app.database.set_priority(undated, Priority::Low).unwrap();
        app.database
            .set_deadline(undated, Some("2030-01-01 12:00"))
            .unwrap();
        app.reload(Some(undated)).unwrap();
        assert_eq!(
            ids(&app),
            [undated, later, earlier, child, grandchild, sibling]
        );
        app.config
            .as_mut()
            .unwrap()
            .save_sort_order(SortOrder::Priority)
            .unwrap();
        app.database
            .set_deadline(later, Some("2000-01-01 00:00"))
            .unwrap();
        app.reload(Some(undated)).unwrap();
        assert_eq!(
            ids(&app),
            [later, undated, earlier, sibling, child, grandchild]
        );

        app.config
            .as_mut()
            .unwrap()
            .save_sort_order(SortOrder::Deadline)
            .unwrap();
        app.reload(Some(undated)).unwrap();
        app.apply(VimAction::MoveTask(TaskMove::Indent, 1)).unwrap();
        assert_eq!(app.selected_todo().unwrap().parent_id, Some(later));
        assert_eq!(app.sort_order(), SortOrder::Deadline);
        app.apply(VimAction::MoveTask(TaskMove::Outdent, 1))
            .unwrap();
        assert_eq!(app.selected_todo().unwrap().parent_id, None);
        std::fs::remove_dir_all(folder).unwrap();
    }
}

fn ids(app: &App) -> Vec<i64> {
    app.visible_indices()
        .into_iter()
        .map(|index| app.todos[index].id)
        .collect()
}

#[test]
fn priority_indentation_uses_own_priority_order_despite_urgent_descendants() {
    let database = crate::db::Database::memory();
    let low = database.add("Low root", None, Priority::Low).unwrap();
    let child = database
        .add("Urgent child", Some(low), Priority::High)
        .unwrap();
    let mid = database.add("Mid root", None, Priority::Mid).unwrap();
    let high = database.add("High root", None, Priority::High).unwrap();
    let mut app = App::new(database).unwrap();
    app.select_id(low);
    assert_eq!(ids(&app), [high, mid, low, child]);

    app.apply(VimAction::MoveTask(TaskMove::Indent, 1)).unwrap();
    assert_eq!(app.selected_todo().unwrap().id, low);
    assert_eq!(app.selected_todo().unwrap().parent_id, Some(mid));
    assert_eq!(ids(&app), [high, mid, low, child]);
    assert_eq!(
        app.todos
            .iter()
            .find(|todo| todo.id == child)
            .unwrap()
            .parent_id,
        Some(low)
    );

    app.apply(VimAction::MoveTask(TaskMove::Outdent, 1))
        .unwrap();
    assert_eq!(app.selected_todo().unwrap().parent_id, None);
    assert_eq!(ids(&app), [high, mid, low, child]);
    assert_eq!(app.sort_order(), SortOrder::Priority);
}
