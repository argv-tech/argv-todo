use anyhow::Result;
use std::collections::HashMap;

use super::App;
use crate::{
    config::SortOrder,
    db::{Priority, TaskMove, Todo},
};

impl App {
    pub(crate) fn branch_priorities(&self) -> HashMap<i64, Priority> {
        let mut branches: HashMap<_, _> = self
            .todos
            .iter()
            .map(|todo| (todo.id, (todo.parent_id, todo.priority)))
            .collect();
        if self.sort_order() == SortOrder::Priority {
            for todo in &self.todos {
                let mut ancestor = todo.parent_id;
                while let Some((parent, priority)) = ancestor.and_then(|id| branches.get_mut(&id)) {
                    // An equally urgent ancestor already carries this priority upward.
                    if *priority <= todo.priority {
                        break;
                    }
                    *priority = todo.priority;
                    ancestor = *parent;
                }
            }
        }
        branches
            .into_iter()
            .map(|(id, (_, priority))| (id, priority))
            .collect()
    }

    pub(super) fn sibling_sort_key(
        &self,
        todo: &Todo,
        branch_priorities: &HashMap<i64, Priority>,
    ) -> (Option<Priority>, i64, i64) {
        let priority = match self.sort_order() {
            SortOrder::Priority => Some(branch_priorities[&todo.id]),
            SortOrder::Manual => None,
        };
        (priority, todo.position, todo.id)
    }

    pub(super) fn move_task(&mut self, movement: TaskMove, count: usize) -> Result<()> {
        if self.sort_order() != SortOrder::Manual
            && matches!(movement, TaskMove::Up | TaskMove::Down)
        {
            self.message("Choose sort_order = manual in configuration to move tasks up or down.");
            return Ok(());
        }
        let Some(id) = self.selected_todo().map(|todo| todo.id) else {
            return Ok(());
        };
        let changed = if self.sort_order() == SortOrder::Priority {
            let priorities = self.branch_priorities();
            let mut tasks: Vec<_> = self.todos.iter().collect();
            tasks.sort_by_key(|todo| self.sibling_sort_key(todo, &priorities));
            let sibling_order: Vec<_> = tasks.into_iter().map(|todo| todo.id).collect();
            self.database
                .move_task_with_sibling_order(id, movement, count, &sibling_order)?
        } else {
            self.database.move_task(id, movement, count)?
        };
        if changed {
            self.reload(Some(id))?;
            self.message("Task tree moved.");
        }
        Ok(())
    }
}
