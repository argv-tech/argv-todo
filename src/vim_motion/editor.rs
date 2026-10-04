use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::vim_action::{Motion, VimAction};

/// Byte cursor always sits on a grapheme boundary, including after a paste.
#[derive(Default)]
pub(crate) struct Editor {
    text: String,
    cursor: usize,
}

impl Editor {
    pub(crate) fn new(text: String) -> Self {
        let cursor = text.len();
        Self { text, cursor }
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    fn previous(&self) -> usize {
        self.text[..self.cursor]
            .grapheme_indices(true)
            .next_back()
            .map_or(0, |(i, _)| i)
    }

    fn next(&self) -> usize {
        self.text[self.cursor..]
            .graphemes(true)
            .next()
            .map_or(self.cursor, |g| self.cursor + g.len())
    }

    pub(crate) fn insert(&mut self, text: &str) {
        let clean: String = text
            .chars()
            .map(|c| if c.is_control() { ' ' } else { c })
            .collect();
        self.text.insert_str(self.cursor, &clean);
        self.cursor += clean.len();
        self.normalize_cursor();
    }

    fn normalize_cursor(&mut self) {
        // Inserting or deleting can join graphemes on either side of the cursor.
        self.cursor = self
            .text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .find(|&i| i >= self.cursor)
            .unwrap_or(self.text.len());
    }

    pub(crate) fn apply(&mut self, action: VimAction) {
        match action {
            VimAction::Insert(c) => self.insert(&c.to_string()),
            VimAction::Clear => {
                self.text.clear();
                self.cursor = 0;
            }
            VimAction::Backspace => {
                let previous = self.previous();
                self.text.replace_range(previous..self.cursor, "");
                self.cursor = previous;
            }
            VimAction::DeleteChar => {
                self.text.replace_range(self.cursor..self.next(), "");
            }
            VimAction::DeleteWord => {
                let end = self.cursor;
                self.word_backward();
                self.text.replace_range(self.cursor..end, "");
            }
            VimAction::Move(motion, count) => {
                for _ in 0..count {
                    match motion {
                        Motion::Left => self.cursor = self.previous(),
                        Motion::Right => self.cursor = self.next(),
                        Motion::LineStart => self.cursor = 0,
                        Motion::LineEnd => self.cursor = self.text.len(),
                        Motion::WordBackward => self.word_backward(),
                        Motion::WordForward => self.word_forward(),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        self.normalize_cursor();
    }

    fn word_backward(&mut self) {
        while self.cursor > 0 && self.text[self.previous()..self.cursor].trim().is_empty() {
            self.cursor = self.previous();
        }
        while self.cursor > 0 && !self.text[self.previous()..self.cursor].trim().is_empty() {
            self.cursor = self.previous();
        }
    }

    fn word_forward(&mut self) {
        while self.cursor < self.text.len()
            && !self.text[self.cursor..self.next()].trim().is_empty()
        {
            self.cursor = self.next();
        }
        while self.cursor < self.text.len() && self.text[self.cursor..self.next()].trim().is_empty()
        {
            self.cursor = self.next();
        }
    }

    /// Returns visible text and cursor column without splitting wide graphemes.
    pub(crate) fn viewport(&self, width: u16) -> (&str, u16) {
        if width == 0 {
            return ("", 0);
        }
        let mut start = 0;
        while UnicodeWidthStr::width(&self.text[start..self.cursor]) >= usize::from(width) {
            let Some(g) = self.text[start..self.cursor].graphemes(true).next() else {
                break;
            };
            start += g.len();
        }
        let column = UnicodeWidthStr::width(&self.text[start..self.cursor]) as u16;
        (&self.text[start..], column)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/vim_motion/editor.rs"]
mod tests;
