use anyhow::{Context, Result};

use super::App;
use crate::vim_motion::InputTarget;

impl App {
    pub(super) fn begin_note(&mut self) {
        let Some(todo) = self.selected_todo() else {
            self.message("Select a task to add a note.");
            return;
        };
        let id = todo.id;
        self.editor.reset_multiline(todo.note.clone());
        self.editing_id = Some(id);
        self.vim.begin_input(InputTarget::Note);
        self.message(
            "Editing note. Enter adds a line. Ctrl-s saves. Esc enters Normal; Esc again cancels.",
        );
    }

    pub(super) fn save_note(&mut self) -> Result<()> {
        let id = self.editing_id.context("No task selected for this note")?;
        let text = self.editor.text();
        self.database
            .update_note(id, if text.trim().is_empty() { "" } else { text })?;
        self.vim.end_input();
        self.editing_id = None;
        self.editor.reset(String::new());
        self.reload(Some(id))?;
        self.message("Note saved.");
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/app/notes.rs"]
mod tests;
