use std::collections::{HashMap, HashSet};

use super::{App, TreeRow};
use crate::{db::Todo, vim_motion::VimMode};

impl App {
    /// All tasks in tree order. Searches retain the ancestors of matching tasks.
    pub(crate) fn visible_rows(&self) -> Vec<TreeRow> {
        let query = if self.vim.mode() == VimMode::Search {
            self.editor.text()
        } else {
            &self.query
        }
        .to_lowercase();
        let by_id: HashMap<_, _> = self
            .todos
            .iter()
            .enumerate()
            .map(|(index, todo)| (todo.id, index))
            .collect();
        let mut children: HashMap<Option<i64>, Vec<usize>> = HashMap::new();
        let mut included = HashSet::new();
        for (index, todo) in self.todos.iter().enumerate() {
            let parent = todo.parent_id.filter(|id| by_id.contains_key(id));
            children.entry(parent).or_default().push(index);
            if query.is_empty() || todo.title.to_lowercase().contains(&query) {
                let mut ancestor = Some(index);
                while let Some(index) = ancestor {
                    if !included.insert(index) {
                        break;
                    }
                    ancestor = self.todos[index]
                        .parent_id
                        .and_then(|id| by_id.get(&id).copied());
                }
            }
        }
        let mut stack: Vec<_> = children
            .get(&None)
            .into_iter()
            .flatten()
            .rev()
            .map(|&index| TreeRow { index, depth: 0 })
            .collect();
        let mut rows = Vec::new();
        while let Some(row) = stack.pop() {
            if !included.contains(&row.index) {
                continue;
            }
            if let Some(children) = children.get(&Some(self.todos[row.index].id)) {
                stack.extend(children.iter().rev().map(|&index| TreeRow {
                    index,
                    depth: row.depth + 1,
                }));
            }
            rows.push(row);
        }
        rows
    }

    pub(super) fn visible_indices(&self) -> Vec<usize> {
        self.visible_rows()
            .into_iter()
            .map(|row| row.index)
            .collect()
    }

    pub(crate) fn child_counts(&self, id: i64) -> (usize, usize) {
        self.todos
            .iter()
            .filter(|todo| todo.parent_id == Some(id))
            .fold((0, 0), |(done, total), todo| {
                (done + usize::from(todo.done), total + 1)
            })
    }

    pub(super) fn select_id(&mut self, id: i64) {
        self.list.select(
            self.visible_indices()
                .iter()
                .position(|&index| self.todos[index].id == id),
        );
        self.normalize_selection();
    }

    pub(super) fn select_child(&mut self) {
        let Some(id) = self.selected_todo().map(|todo| todo.id) else {
            return;
        };
        if let Some(child) = self
            .todos
            .iter()
            .find(|todo| todo.parent_id == Some(id))
            .map(|todo| todo.id)
        {
            self.query.clear();
            self.select_id(child);
            self.message("Child selected.");
        }
    }

    pub(super) fn select_parent(&mut self) -> bool {
        let Some(id) = self.selected_todo().and_then(|todo| todo.parent_id) else {
            return false;
        };
        self.query.clear();
        self.select_id(id);
        self.message("Parent selected.");
        true
    }

    pub(super) fn selected_todo(&self) -> Option<&Todo> {
        self.list
            .selected()
            .and_then(|row| self.visible_indices().get(row).copied())
            .map(|index| &self.todos[index])
    }

    pub(super) fn normalize_selection(&mut self) {
        let len = self.visible_indices().len();
        self.list.select(if len == 0 {
            None
        } else {
            Some(self.list.selected().unwrap_or(0).min(len - 1))
        });
    }
}
