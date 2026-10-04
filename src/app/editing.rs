use anyhow::Result;

use super::App;
use crate::vim_motion::{InputTarget, VimAction};

impl App {
    pub(super) fn begin_add(&mut self, parent_id: Option<i64>) {
        let selected_id = self.selected_todo().map(|todo| todo.id);
        // Show the full tree so the draft and saved task occupy the same place.
        self.query.clear();
        if let Some(id) = parent_id {
            self.expand_task(id);
        }
        self.adding_parent = parent_id;
        self.adding_relative = None;
        self.editing_id = None;
        self.input_priority = parent_id
            .and_then(|id| self.todos.iter().find(|todo| todo.id == id))
            .map_or(self.default_priority(), |todo| todo.priority);
        self.editor.reset(String::new());
        self.vim.begin_input(InputTarget::Task);
        if let Some(id) = selected_id {
            self.select_id(id);
        }
        self.message(if parent_id.is_some() {
            "New child task. Enter saves. Esc enters Normal; Esc again cancels."
        } else {
            "Enter saves. Esc enters Normal; Esc again cancels."
        });
    }

    pub(super) fn begin_sibling(&mut self, above: bool) {
        let selected = self
            .selected_todo()
            .map(|todo| (todo.id, todo.parent_id, todo.priority));
        self.begin_add(selected.and_then(|(_, parent, _)| parent));
        if let Some((id, _, priority)) = selected {
            self.adding_relative = Some((id, above));
            self.input_priority = priority;
        }
    }

    pub(super) fn apply_input(&mut self, action: VimAction) -> Result<()> {
        match action {
            VimAction::Cancel => {
                self.vim.end_input();
                self.editor.reset(String::new());
                self.editing_id = None;
                self.adding_parent = None;
                self.adding_relative = None;
                self.normalize_selection();
                self.message("Cancelled.");
            }
            VimAction::Submit => {
                self.editor.prepare_submit();
                let text = self.editor.text().trim().to_owned();
                if self.vim.input() == Some(InputTarget::Search) {
                    self.query = text;
                    self.vim.end_input();
                    self.editor.reset(String::new());
                    self.normalize_selection();
                    self.message(if self.query.is_empty() {
                        "Search cleared."
                    } else {
                        "Search applied. Esc clears it."
                    });
                } else {
                    anyhow::ensure!(!text.is_empty(), "Task title cannot be empty");
                    let id = if let Some(id) = self.editing_id {
                        self.database.update(id, &text, self.input_priority)?;
                        id
                    } else if let Some((id, above)) = self.adding_relative {
                        self.database
                            .add_relative(&text, id, above, self.input_priority)?
                    } else {
                        self.database
                            .add(&text, self.adding_parent, self.input_priority)?
                    };
                    let was_edit = self.editing_id.is_some();
                    if !was_edit {
                        self.query.clear();
                    }
                    self.vim.end_input();
                    self.editing_id = None;
                    self.adding_parent = None;
                    self.adding_relative = None;
                    self.editor.reset(String::new());
                    self.reload(Some(id))?;
                    self.message(if was_edit {
                        "Task updated."
                    } else {
                        "Task added."
                    });
                }
            }
            VimAction::CyclePriority if self.vim.input() == Some(InputTarget::Task) => {
                self.input_priority = self.input_priority.next();
            }
            _ => {
                self.editor.apply(action);
                self.vim.set_mode(self.editor.mode());
                if self.vim.input() == Some(InputTarget::Search) {
                    self.normalize_selection();
                }
            }
        }
        Ok(())
    }
}
