use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::{Editor, TextMode};

impl Editor {
    pub(super) fn line_start(&self, position: usize) -> usize {
        if self.text_mode == TextMode::SingleLine {
            return 0;
        }
        self.text[..position]
            .rfind('\n')
            .map_or(0, |start| start + 1)
    }

    pub(super) fn line_end(&self, position: usize) -> usize {
        if self.text_mode == TextMode::SingleLine {
            return self.text.len();
        }
        self.text[position..]
            .find('\n')
            .map_or(self.text.len(), |end| position + end)
    }

    pub(super) fn line_range(&self, position: usize, count: usize) -> Range<usize> {
        let start = self.line_start(position);
        let mut end = start;
        for _ in 0..count.max(1) {
            end = (self.line_end(end) + 1).min(self.text.len());
            if end == self.text.len() {
                break;
            }
        }
        start..end
    }

    pub(super) fn vertical_position(&mut self, down: bool, count: usize) -> usize {
        let column = self
            .preferred_column
            .unwrap_or_else(|| self.text[self.line_start(self.cursor)..self.cursor].width());
        self.preferred_column = Some(column);
        let mut start = self.line_start(self.cursor);
        for _ in 0..count {
            if down {
                let end = self.line_end(start);
                if end == self.text.len() {
                    break;
                }
                start = end + 1;
            } else if start > 0 {
                start = self.line_start(start - 1);
            } else {
                break;
            }
        }
        self.position_at_column(start, column)
    }

    pub(super) fn position_at_column(&self, start: usize, column: usize) -> usize {
        let end = self.line_end(start);
        let mut used = 0;
        for (byte, grapheme) in self.text[start..end].grapheme_indices(true) {
            if used + grapheme.width() > column {
                return start + byte;
            }
            used += grapheme.width();
        }
        end
    }

    pub(super) fn clean_text(&self, text: &str) -> String {
        let text = if self.text_mode == TextMode::Multiline {
            text.replace("\r\n", "\n").replace('\r', "\n")
        } else {
            text.to_owned()
        };
        text.chars()
            .map(|character| {
                if character.is_control()
                    && !(character == '\n' && self.text_mode == TextMode::Multiline)
                {
                    ' '
                } else {
                    character
                }
            })
            .collect()
    }

    pub(super) fn leave_typing(&mut self) {
        if self.mode.typing() && self.cursor > self.line_start(self.cursor) {
            self.cursor = self.previous_at(self.cursor);
        }
    }
}
