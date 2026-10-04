use super::{App, TaskPane, TreeRow};

const PANES: [TaskPane; 3] = [TaskPane::Tree, TaskPane::Todo, TaskPane::Completed];

impl App {
    pub(crate) fn is_collapsed(&self, id: i64) -> bool {
        self.collapsed.contains(&id)
    }

    pub(super) fn expanded_rows(&self, rows: Vec<TreeRow>) -> Vec<TreeRow> {
        let mut collapsed_depth = None;
        rows.into_iter()
            .filter(|row| {
                if collapsed_depth.is_some_and(|depth| row.depth > depth) {
                    return false;
                }
                collapsed_depth = self
                    .is_collapsed(self.todos[row.index].id)
                    .then_some(row.depth);
                true
            })
            .collect()
    }

    pub(super) fn expand_ancestors(&mut self, id: i64) {
        let parent = self
            .todos
            .iter()
            .find(|todo| todo.id == id)
            .and_then(|todo| todo.parent_id);
        if let Some(id) = parent {
            self.expand_task(id);
        }
    }

    pub(super) fn expand_task(&mut self, id: i64) {
        let selections = self.pane_selections();
        let mut parent = Some(id);
        let mut changed = false;
        while let Some(id) = parent {
            changed |= self.collapsed.remove(&id);
            parent = self
                .todos
                .iter()
                .find(|todo| todo.id == id)
                .and_then(|todo| todo.parent_id);
        }
        if changed {
            self.restore_pane_selections(selections);
        }
    }

    pub(super) fn toggle_collapse(&mut self) {
        let Some(id) = self.selected_todo().map(|todo| todo.id) else {
            return;
        };
        if self.child_counts(id).1 == 0 {
            return;
        }
        let selections = self.pane_selections();
        let collapsed = if self.collapsed.remove(&id) {
            false
        } else {
            self.collapsed.insert(id);
            true
        };
        self.restore_pane_selections(selections);
        self.message(if collapsed {
            "Task collapsed."
        } else {
            "Task expanded."
        });
    }

    fn pane_selections(&self) -> [Option<i64>; 3] {
        PANES.map(|pane| {
            self.pane_list(pane).selected().and_then(|selected| {
                self.rows_for_pane(pane)
                    .into_iter()
                    .filter(|row| !row.ghost)
                    .nth(selected)
                    .map(|row| self.todos[row.index].id)
            })
        })
    }

    fn restore_pane_selections(&mut self, selections: [Option<i64>; 3]) {
        // Preserve each pane's selected task when folding changes row positions.
        for (pane, selected_id) in PANES.into_iter().zip(selections) {
            let rows = self.rows_for_pane(pane);
            let mut candidate = selected_id;
            let mut selected = None;
            while let Some(id) = candidate {
                selected = rows
                    .iter()
                    .filter(|row| !row.ghost)
                    .position(|row| self.todos[row.index].id == id);
                if selected.is_some() {
                    break;
                }
                candidate = self
                    .todos
                    .iter()
                    .find(|todo| todo.id == id)
                    .and_then(|todo| todo.parent_id);
            }
            self.pane_list_mut(pane).select(selected);
        }
        self.normalize_selection();
    }
}

#[cfg(test)]
#[path = "../../tests/unit/app/folding.rs"]
mod tests;
