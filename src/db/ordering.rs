use std::collections::{BTreeMap, HashMap};

use anyhow::{Context, Result};
use rusqlite::params;

use super::Database;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TaskMove {
    Up,
    Down,
    Outdent,
    Indent,
}

impl Database {
    pub(crate) fn move_task(&mut self, id: i64, movement: TaskMove, count: usize) -> Result<bool> {
        let transaction = self.connection.transaction()?;
        let tasks = {
            let mut statement = transaction.prepare(
                "SELECT id, title, done, created_at, updated_at, parent_id, priority, position
                 FROM todos ORDER BY position, id",
            )?;
            statement
                .query_map([], Self::todo_from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        let mut parents: HashMap<_, _> =
            tasks.iter().map(|task| (task.id, task.parent_id)).collect();
        let old_parent = *parents
            .get(&id)
            .context("Task no longer exists; press Ctrl-r to refresh")?;
        let mut siblings: BTreeMap<Option<i64>, Vec<i64>> = BTreeMap::new();
        for task in &tasks {
            siblings.entry(task.parent_id).or_default().push(task.id);
        }
        let original = siblings.clone();
        match movement {
            TaskMove::Up | TaskMove::Down => {
                let group = siblings
                    .get_mut(&old_parent)
                    .context("Task sibling order is unavailable")?;
                let index = group
                    .iter()
                    .position(|&task| task == id)
                    .context("Task is missing from its siblings")?;
                let target = if movement == TaskMove::Up {
                    index.saturating_sub(count)
                } else {
                    index.saturating_add(count).min(group.len() - 1)
                };
                group.remove(index);
                group.insert(target, id);
            }
            TaskMove::Outdent | TaskMove::Indent => {
                for _ in 0..count {
                    if !change_parent(&mut siblings, &mut parents, id, movement)? {
                        break;
                    }
                }
            }
        }
        let changed = siblings != original;
        if changed {
            transaction.execute(
                "UPDATE todos SET parent_id = ?1, updated_at = strftime('%Y-%m-%d %H:%M:%S', 'now') WHERE id = ?2",
                params![parents[&id], id],
            )?;
            for (parent, group) in &siblings {
                if original.get(parent) == Some(group) {
                    continue;
                }
                for (position, &task) in group.iter().enumerate() {
                    transaction.execute(
                        "UPDATE todos SET position = ?1 WHERE id = ?2",
                        params![position as i64 + 1, task],
                    )?;
                }
            }
        }
        transaction.commit()?;
        Ok(changed)
    }
}

fn change_parent(
    siblings: &mut BTreeMap<Option<i64>, Vec<i64>>,
    parents: &mut HashMap<i64, Option<i64>>,
    id: i64,
    movement: TaskMove,
) -> Result<bool> {
    let parent = parents[&id];
    let group = siblings
        .get(&parent)
        .context("Task sibling order is unavailable")?;
    let index = group
        .iter()
        .position(|&task| task == id)
        .context("Task is missing from its siblings")?;
    let (new_parent, insertion) = if movement == TaskMove::Indent {
        let Some(previous) = index.checked_sub(1).map(|index| group[index]) else {
            return Ok(false);
        };
        (
            Some(previous),
            siblings.get(&Some(previous)).map_or(0, Vec::len),
        )
    } else {
        let Some(parent) = parent else {
            return Ok(false);
        };
        let grandparent = *parents
            .get(&parent)
            .context("Parent task no longer exists")?;
        let position = siblings
            .get(&grandparent)
            .and_then(|group| group.iter().position(|&id| id == parent))
            .context("Parent is missing from its siblings")?;
        (grandparent, position + 1)
    };
    // A sibling or ancestor becomes the new parent; descendants keep their links.
    siblings
        .get_mut(&parent)
        .context("Task sibling order is unavailable")?
        .remove(index);
    siblings
        .entry(new_parent)
        .or_default()
        .insert(insertion, id);
    parents.insert(id, new_parent);
    Ok(true)
}

#[cfg(test)]
#[path = "../../tests/unit/db/ordering.rs"]
mod tests;
