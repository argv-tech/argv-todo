use ratatui::widgets::ListState;

use super::{App, TreeRow};
use crate::config::TaskView;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TaskPane {
    #[default]
    Tree,
    Todo,
    Completed,
}

#[derive(Default)]
pub(crate) struct TaskPanes {
    focus: TaskPane,
    todo: ListState,
    completed: ListState,
}

impl App {
    pub(crate) fn task_view(&self) -> TaskView {
        self.config
            .as_ref()
            .map_or(TaskView::Normal, |config| config.task_view)
    }

    pub(crate) fn task_pane(&self) -> TaskPane {
        match (self.task_view(), self.panes.focus) {
            (TaskView::Normal, _) => TaskPane::Tree,
            (TaskView::Split, TaskPane::Completed) => TaskPane::Completed,
            (TaskView::Split, _) => TaskPane::Todo,
        }
    }

    pub(crate) fn pane_list(&self, pane: TaskPane) -> &ListState {
        match pane {
            TaskPane::Tree => &self.list,
            TaskPane::Todo => &self.panes.todo,
            TaskPane::Completed => &self.panes.completed,
        }
    }

    pub(crate) fn pane_list_mut(&mut self, pane: TaskPane) -> &mut ListState {
        match pane {
            TaskPane::Tree => &mut self.list,
            TaskPane::Todo => &mut self.panes.todo,
            TaskPane::Completed => &mut self.panes.completed,
        }
    }

    pub(super) fn active_list(&self) -> &ListState {
        self.pane_list(self.task_pane())
    }

    pub(super) fn active_list_mut(&mut self) -> &mut ListState {
        self.pane_list_mut(self.task_pane())
    }

    pub(crate) fn rows_for_pane(&self, pane: TaskPane) -> Vec<TreeRow> {
        let rows = self.visible_rows();
        let rows = if pane == TaskPane::Tree && self.show_completed() {
            rows
        } else {
            self.rows_by_completion(rows, pane == TaskPane::Completed)
        };
        self.expanded_rows(rows)
    }

    pub(super) fn switch_task_pane(&mut self) {
        if self.task_view() == TaskView::Normal {
            return;
        }
        self.normalize_selection();
        self.panes.focus = if self.task_pane() == TaskPane::Todo {
            TaskPane::Completed
        } else {
            TaskPane::Todo
        };
        self.message("");
    }

    pub(super) fn select_in_task_view(&mut self, id: i64) {
        let pane = if self.task_view() == TaskView::Split {
            if self.todos.iter().any(|todo| todo.id == id && todo.done) {
                TaskPane::Completed
            } else {
                TaskPane::Todo
            }
        } else {
            TaskPane::Tree
        };
        self.panes.focus = pane;
        let row = self
            .rows_for_pane(pane)
            .iter()
            .filter(|row| !row.ghost)
            .position(|row| self.todos[row.index].id == id);
        self.pane_list_mut(pane).select(row);
    }

    pub(super) fn reconcile_task_view(&mut self, selected_id: Option<i64>) {
        if let Some(id) = selected_id {
            self.select_id(id);
        }
        self.normalize_selection();
    }

    pub(super) fn normalize_panes(&mut self) {
        for pane in [TaskPane::Tree, TaskPane::Todo, TaskPane::Completed] {
            let len = self
                .rows_for_pane(pane)
                .iter()
                .filter(|row| !row.ghost)
                .count();
            normalize_list(self.pane_list_mut(pane), len);
        }
    }

    fn rows_by_completion(&self, rows: Vec<TreeRow>, completed: bool) -> Vec<TreeRow> {
        let mut included = vec![false; rows.len()];
        let mut ancestors = Vec::new();
        for (position, row) in rows.iter().enumerate() {
            ancestors.truncate(row.depth);
            if self.todos[row.index].done == completed {
                included[position] = true;
                if completed {
                    for &ancestor in &ancestors {
                        included[ancestor] = true;
                    }
                }
            }
            ancestors.push(position);
        }
        let mut ancestors = Vec::new();
        let mut filtered = Vec::new();
        for (row, included) in rows.into_iter().zip(included) {
            ancestors.truncate(row.depth);
            if included {
                filtered.push(TreeRow {
                    index: row.index,
                    depth: ancestors.iter().filter(|&&included| included).count(),
                    ghost: self.todos[row.index].done != completed,
                });
            }
            ancestors.push(included);
        }
        filtered
    }
}

fn normalize_list(list: &mut ListState, len: usize) {
    list.select(if len == 0 {
        None
    } else {
        Some(list.selected().unwrap_or(0).min(len - 1))
    });
}

#[cfg(test)]
#[path = "../../tests/unit/app/views.rs"]
mod tests;
