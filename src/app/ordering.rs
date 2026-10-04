use anyhow::Result;

use super::App;
use crate::{
    config::SortOrder,
    db::{Priority, TaskMove, Todo},
};

impl App {
    pub(super) fn sibling_sort_key(&self, todo: &Todo) -> (Option<Priority>, i64, i64) {
        let priority = match self.sort_order() {
            SortOrder::Priority => Some(todo.priority),
            SortOrder::Manual => None,
        };
        (priority, todo.position, todo.id)
    }

    pub(super) fn move_task(&mut self, movement: TaskMove, count: usize) -> Result<()> {
        if self.sort_order() != SortOrder::Manual {
            self.message("Choose sort_order = manual in configuration to move tasks.");
            return Ok(());
        }
        let Some(id) = self.selected_todo().map(|todo| todo.id) else {
            return Ok(());
        };
        if self.database.move_task(id, movement, count)? {
            self.reload(Some(id))?;
            self.message("Task tree moved.");
        }
        Ok(())
    }
}
