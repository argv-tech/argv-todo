use super::{
    EditAction, EditTarget, Editor, InsertPosition, Motion, Operator, Snapshot, VimAction, VimMode,
};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

impl Editor {
    pub(super) fn apply_inner(&mut self, action: VimAction) {
        match action {
            VimAction::Insert(c) => {
                if self.mode == VimMode::Replace {
                    self.replace_history.push(Snapshot {
                        text: self.text.clone(),
                        cursor: self.cursor,
                    });
                    let end = self.next_at(self.cursor);
                    self.text
                        .replace_range(self.cursor..end, &clean_text(&c.to_string()));
                    self.cursor += clean_text(&c.to_string()).len();
                } else {
                    self.insert_raw(&c.to_string());
                }
            }
            VimAction::Clear => {
                self.text.clear();
                self.cursor = 0;
            }
            VimAction::Backspace => {
                if self.mode == VimMode::Replace {
                    if let Some(snapshot) = self.replace_history.pop() {
                        self.text = snapshot.text;
                        self.cursor = snapshot.cursor;
                    }
                } else {
                    let start = self.previous_at(self.cursor);
                    self.text.replace_range(start..self.cursor, "");
                    self.cursor = start;
                }
            }
            VimAction::DeleteChar => {
                self.text
                    .replace_range(self.cursor..self.next_at(self.cursor), "");
            }
            VimAction::DeleteWord => {
                let start = self.word_backward(self.cursor, true);
                self.text.replace_range(start..self.cursor, "");
                self.cursor = start;
            }
            VimAction::Move(motion, count) | VimAction::Input(EditAction::Move(motion, count)) => {
                if let Some(position) = self.motion_position(motion, count) {
                    self.cursor = position;
                }
                self.replace_history.clear();
            }
            VimAction::Input(action) => self.edit(action),
            _ => {}
        }
    }

    fn edit(&mut self, action: EditAction) {
        match action {
            EditAction::Normal => {
                if self.mode.typing() {
                    self.cursor = self.previous_at(self.cursor);
                }
                self.mode = VimMode::Normal;
                self.anchor = None;
            }
            EditAction::Insert(position, count) => {
                self.cursor = match position {
                    InsertPosition::Cursor => self.cursor,
                    InsertPosition::After => self.next_at(self.cursor),
                    InsertPosition::FirstNonBlank => self
                        .motion_position(Motion::FirstNonBlank, 1)
                        .unwrap_or(self.cursor),
                    InsertPosition::End => self.text.len(),
                };
                self.mode = VimMode::Insert;
                self.anchor = None;
                self.insert_count = count;
            }
            EditAction::Visual(line) => {
                let mode = if line {
                    VimMode::VisualLine
                } else {
                    VimMode::Visual
                };
                if self.mode == mode {
                    self.mode = VimMode::Normal;
                    self.anchor = None;
                } else {
                    self.anchor = Some(self.anchor.unwrap_or(self.cursor));
                    self.mode = mode;
                }
            }
            EditAction::SwapAnchor => {
                if let Some(anchor) = &mut self.anchor {
                    std::mem::swap(anchor, &mut self.cursor);
                }
            }
            EditAction::Select(object, around, count) => {
                if let Some(range) = self.object_range(object, around, count) {
                    if range.is_empty() {
                        self.mode = VimMode::Normal;
                        self.anchor = None;
                        return;
                    }
                    self.mode = VimMode::Visual;
                    self.anchor = Some(range.start);
                    self.cursor = self.previous_at(range.end);
                }
            }
            EditAction::Operate(operator, target, count) => {
                let range = match target {
                    EditTarget::Line => Some(0..self.text.len()),
                    EditTarget::Selection => self.selection(),
                    EditTarget::Object(object, around) => self.object_range(object, around, count),
                    EditTarget::Motion(motion) => self.motion_range(motion, operator, count),
                };
                if let Some(range) = range {
                    self.operate(operator, range);
                }
            }
            EditAction::Replace(character, count) => {
                let range = if self.mode.visual() {
                    self.selection()
                } else {
                    self.text[self.cursor..]
                        .graphemes(true)
                        .nth(count.saturating_sub(1))
                        .map(|g| {
                            let length: usize = self.text[self.cursor..]
                                .graphemes(true)
                                .take(count.saturating_sub(1))
                                .map(str::len)
                                .sum();
                            self.cursor..self.cursor + length + g.len()
                        })
                };
                if let Some(range) = range {
                    let replacement = clean_text(&character.to_string())
                        .repeat(self.text[range.clone()].graphemes(true).count());
                    let start = range.start;
                    self.text.replace_range(range, &replacement);
                    self.cursor = start
                        + replacement
                            .grapheme_indices(true)
                            .next_back()
                            .map_or(0, |(i, _)| i);
                    self.mode = VimMode::Normal;
                    self.anchor = None;
                }
            }
            EditAction::ReplaceMode => {
                self.mode = VimMode::Replace;
                self.anchor = None;
                self.replace_history.clear();
            }
            EditAction::Put(before, count) => self.put(before, count),
            _ => {}
        }
    }

    fn operate(&mut self, operator: Operator, range: Range<usize>) {
        let start = range.start;
        match operator {
            Operator::Delete | Operator::Change | Operator::Yank => {
                if !range.is_empty() {
                    self.register = self.text[range.clone()].to_owned();
                }
                if operator != Operator::Yank {
                    self.text.replace_range(range, "");
                }
                self.mode = if operator == Operator::Change {
                    VimMode::Insert
                } else {
                    VimMode::Normal
                };
            }
            Operator::Lowercase | Operator::Uppercase | Operator::ToggleCase => {
                let replacement: String = self.text[range.clone()]
                    .chars()
                    .flat_map(|c| {
                        if operator == Operator::Uppercase
                            || (operator == Operator::ToggleCase && c.is_lowercase())
                        {
                            c.to_uppercase().collect::<Vec<_>>()
                        } else {
                            c.to_lowercase().collect::<Vec<_>>()
                        }
                    })
                    .collect();
                self.text.replace_range(range, &replacement);
                self.mode = VimMode::Normal;
            }
        }
        self.cursor = start;
        self.anchor = None;
    }

    fn put(&mut self, before: bool, count: usize) {
        if self.register.is_empty() {
            return;
        }
        let text = self.register.repeat(count);
        if let Some(range) = self.selection() {
            self.cursor = range.start;
            self.text.replace_range(range, "");
        } else if !before {
            self.cursor = self.next_at(self.cursor);
        }
        self.mode = VimMode::Insert;
        self.anchor = None;
        self.insert_raw(&text);
        self.cursor = self.previous_at(self.cursor);
        self.mode = VimMode::Normal;
    }

    pub(super) fn paste_inner(&mut self, text: &str) {
        if self.mode == VimMode::Replace {
            for character in clean_text(text).chars() {
                self.apply_inner(VimAction::Insert(character));
                self.normalize_cursor();
            }
        } else if self.mode.typing() {
            self.insert_raw(text);
        } else {
            if let Some(range) = self.selection() {
                self.cursor = range.start;
                self.text.replace_range(range, "");
            } else {
                self.cursor = self.next_at(self.cursor);
            }
            self.mode = VimMode::Insert;
            self.anchor = None;
            self.insert_raw(text);
            self.normalize_cursor();
            self.cursor = self.previous_at(self.cursor);
            self.mode = VimMode::Normal;
        }
    }

    fn insert_raw(&mut self, text: &str) {
        let clean = clean_text(text);
        self.text.insert_str(self.cursor, &clean);
        self.cursor += clean.len();
    }
}

fn clean_text(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}
