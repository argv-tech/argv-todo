use anyhow::Result;
use std::collections::HashMap;

use super::App;
use crate::{
    config::SortOrder,
    db::{Priority, TaskMove, Todo},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TaskSortValue<'a> {
    Priority(Priority),
    Deadline(&'a str),
    Undated,
    Manual,
}

#[cfg(test)]
#[path = "../../tests/unit/app/ordering.rs"]
mod tests;

impl App {
    fn task_sort_value<'a>(&self, todo: &'a Todo) -> TaskSortValue<'a> {
        match self.sort_order() {
            SortOrder::Priority => TaskSortValue::Priority(todo.priority),
            SortOrder::Deadline => todo
                .deadline
                .as_deref()
                .map_or(TaskSortValue::Undated, TaskSortValue::Deadline),
            SortOrder::Manual => TaskSortValue::Manual,
        }
    }

    pub(crate) fn draft_sort_value(&self) -> TaskSortValue<'_> {
        match self.sort_order() {
            SortOrder::Priority => TaskSortValue::Priority(self.input_priority),
            SortOrder::Deadline => TaskSortValue::Undated,
            SortOrder::Manual => TaskSortValue::Manual,
        }
    }

    pub(crate) fn task_sort_values(&self) -> HashMap<i64, TaskSortValue<'_>> {
        if self.sort_order() != SortOrder::Deadline {
            return self
                .todos
                .iter()
                .map(|todo| (todo.id, self.task_sort_value(todo)))
                .collect();
        }

        // Only deadline sorting inherits a value from descendants. Priority
        // sorting compares each sibling's own priority at every tree level.
        let mut branches: HashMap<_, _> = self
            .todos
            .iter()
            .map(|todo| (todo.id, (todo.parent_id, self.task_sort_value(todo))))
            .collect();
        for todo in &self.todos {
            let value = self.task_sort_value(todo);
            let mut ancestor = todo.parent_id;
            while let Some((parent, inherited)) = ancestor.and_then(|id| branches.get_mut(&id)) {
                // An equally early sort value already propagates through this ancestor.
                if *inherited <= value {
                    break;
                }
                *inherited = value;
                ancestor = *parent;
            }
        }
        branches
            .into_iter()
            .map(|(id, (_, value))| (id, value))
            .collect()
    }

    pub(super) fn sibling_sort_key<'a>(
        &self,
        todo: &Todo,
        sort_values: &HashMap<i64, TaskSortValue<'a>>,
    ) -> (TaskSortValue<'a>, i64, i64) {
        (sort_values[&todo.id], todo.position, todo.id)
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
        let changed = if self.sort_order() != SortOrder::Manual {
            let values = self.task_sort_values();
            let mut tasks: Vec<_> = self.todos.iter().collect();
            tasks.sort_by_key(|todo| self.sibling_sort_key(todo, &values));
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
