mod editing;
mod history;
mod motions;
mod objects;
mod viewport;

use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

use super::{
    VimMode,
    vim_action::{EditAction, EditTarget, InsertPosition, Motion, Operator, TextObject, VimAction},
};
use history::{Change, RecordedAction, Snapshot};

/// Byte positions always lie on grapheme boundaries; fields contain one line.
pub(crate) struct Editor {
    text: String,
    cursor: usize,
    mode: VimMode,
    anchor: Option<usize>,
    register: String,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    change: Option<Change>,
    last_change: Vec<RecordedAction>,
    insert_count: usize,
    last_find: Option<Motion>,
    replace_history: Vec<Snapshot>,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl Editor {
    pub(crate) fn new(text: String) -> Self {
        let cursor = text.len();
        Self {
            text,
            cursor,
            mode: VimMode::Insert,
            anchor: None,
            register: String::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            change: None,
            last_change: Vec::new(),
            insert_count: 1,
            last_find: None,
            replace_history: Vec::new(),
        }
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    pub(crate) fn mode(&self) -> VimMode {
        self.mode
    }

    pub(crate) fn reset(&mut self, text: String) {
        let register = std::mem::take(&mut self.register);
        *self = Self::new(text);
        self.register = register;
    }

    pub(crate) fn selection(&self) -> Option<Range<usize>> {
        let anchor = self.anchor?;
        if self.mode == VimMode::VisualLine {
            return Some(0..self.text.len());
        }
        Some(anchor.min(self.cursor)..self.next_at(anchor.max(self.cursor)))
    }

    fn previous_at(&self, position: usize) -> usize {
        self.text[..position]
            .grapheme_indices(true)
            .next_back()
            .map_or(0, |(i, _)| i)
    }

    fn next_at(&self, position: usize) -> usize {
        self.text[position..]
            .graphemes(true)
            .next()
            .map_or(position, |g| position + g.len())
    }

    fn normalize_cursor(&mut self) {
        // Edits may join graphemes on either side of a formerly valid boundary.
        self.cursor = self
            .text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .find(|&i| i >= self.cursor)
            .unwrap_or(self.text.len());
        if !self.mode.typing() && self.cursor == self.text.len() {
            self.cursor = self.previous_at(self.cursor);
        }
        if let Some(anchor) = self.anchor {
            self.anchor = Some(
                self.text
                    .grapheme_indices(true)
                    .map(|(i, _)| i)
                    .find(|&i| i >= anchor)
                    .unwrap_or(self.cursor),
            );
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/vim_motion/editor.rs"]
mod tests;
