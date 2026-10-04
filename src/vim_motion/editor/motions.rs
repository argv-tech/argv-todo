use super::{Editor, Motion, Operator};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum WordClass {
    Space,
    Word,
    Punctuation,
}

impl Editor {
    pub(super) fn class_at(&self, position: usize, big: bool) -> WordClass {
        let grapheme = &self.text[position..self.next_at(position)];
        if grapheme.trim().is_empty() {
            WordClass::Space
        } else if big
            || grapheme
                .chars()
                .next()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            WordClass::Word
        } else {
            WordClass::Punctuation
        }
    }

    pub(super) fn motion_position(&mut self, motion: Motion, count: usize) -> Option<usize> {
        let mut position = self.cursor;
        if motion == Motion::Column {
            let requested = count.saturating_sub(1);
            let mut column = 0;
            for (position, grapheme) in self.text.grapheme_indices(true) {
                column += grapheme.width();
                if column > requested {
                    return Some(position);
                }
            }
            return Some(self.text.len());
        }
        if motion == Motion::MatchingBracket {
            return self.matching_bracket();
        }
        if let Motion::RepeatFind(reverse) = motion {
            let Motion::Find {
                character,
                backward,
                till,
            } = self.last_find?
            else {
                return None;
            };
            return self.find_position(character, backward ^ reverse, till, count, true);
        }
        if let Motion::Find {
            character,
            backward,
            till,
        } = motion
        {
            self.last_find = Some(motion);
            return self.find_position(character, backward, till, count, false);
        }
        for _ in 0..count.min(self.text.graphemes(true).count().saturating_add(1)) {
            position = match motion {
                Motion::Left => self.previous_at(position),
                Motion::Right => self.next_at(position),
                Motion::LineStart => 0,
                Motion::LineEnd => self.text.len(),
                Motion::FirstNonBlank => self
                    .text
                    .grapheme_indices(true)
                    .find(|(_, g)| !g.trim().is_empty())
                    .map_or(0, |(i, _)| i),
                Motion::WordForward | Motion::BigWordForward => {
                    self.word_forward(position, motion == Motion::BigWordForward)
                }
                Motion::WordBackward | Motion::BigWordBackward => {
                    self.word_backward(position, motion == Motion::BigWordBackward)
                }
                Motion::WordEnd | Motion::BigWordEnd => {
                    self.word_end(position, motion == Motion::BigWordEnd)
                }
                Motion::WordEndBackward | Motion::BigWordEndBackward => {
                    self.word_end_backward(position, motion == Motion::BigWordEndBackward)
                }
                _ => position,
            };
        }
        Some(position)
    }

    pub(super) fn word_forward(&self, mut position: usize, big: bool) -> usize {
        if position < self.text.len() {
            let class = self.class_at(position, big);
            while position < self.text.len() && self.class_at(position, big) == class {
                position = self.next_at(position);
            }
        }
        while position < self.text.len() && self.class_at(position, big) == WordClass::Space {
            position = self.next_at(position);
        }
        position
    }

    pub(super) fn word_backward(&self, mut position: usize, big: bool) -> usize {
        position = self.previous_at(position);
        while position > 0 && self.class_at(position, big) == WordClass::Space {
            position = self.previous_at(position);
        }
        let class = self.class_at(position, big);
        while position > 0 && self.class_at(self.previous_at(position), big) == class {
            position = self.previous_at(position);
        }
        position
    }

    fn word_end(&self, position: usize, big: bool) -> usize {
        let mut position = self.next_at(position);
        while position < self.text.len() && self.class_at(position, big) == WordClass::Space {
            position = self.next_at(position);
        }
        if position == self.text.len() {
            return self.previous_at(position);
        }
        let class = self.class_at(position, big);
        while self.next_at(position) < self.text.len()
            && self.class_at(self.next_at(position), big) == class
        {
            position = self.next_at(position);
        }
        position
    }

    fn word_end_backward(&self, mut position: usize, big: bool) -> usize {
        if position == 0 {
            return 0;
        }
        let class = self.class_at(position, big);
        while position > 0 && self.class_at(self.previous_at(position), big) == class {
            position = self.previous_at(position);
        }
        position = self.previous_at(position);
        while position > 0 && self.class_at(position, big) == WordClass::Space {
            position = self.previous_at(position);
        }
        position
    }

    fn find_position(
        &self,
        character: char,
        backward: bool,
        till: bool,
        count: usize,
        repeat: bool,
    ) -> Option<usize> {
        let matches = |g: &str| g.starts_with(character);
        let mut position = self.cursor;
        if repeat && till {
            position = if backward {
                self.previous_at(position)
            } else {
                self.next_at(position)
            };
        }
        for _ in 0..count {
            position = if backward {
                self.text[..position]
                    .grapheme_indices(true)
                    .rev()
                    .find(|(_, g)| matches(g))
                    .map(|(i, _)| i)?
            } else {
                let start = self.next_at(position);
                self.text[start..]
                    .grapheme_indices(true)
                    .find(|(_, g)| matches(g))
                    .map(|(i, _)| start + i)?
            };
        }
        Some(if till {
            if backward {
                self.next_at(position)
            } else {
                self.previous_at(position)
            }
        } else {
            position
        })
    }

    pub(super) fn motion_range(
        &mut self,
        motion: Motion,
        operator: Operator,
        count: usize,
    ) -> Option<Range<usize>> {
        // cw stops at this word's end even when the cursor is on its last character.
        if operator == Operator::Change
            && matches!(motion, Motion::WordForward | Motion::BigWordForward)
            && self.cursor < self.text.len()
            && self.class_at(self.cursor, false) != WordClass::Space
        {
            let big = motion == Motion::BigWordForward;
            let class = self.class_at(self.cursor, big);
            let mut end = self.cursor;
            while self.next_at(end) < self.text.len()
                && self.class_at(self.next_at(end), big) == class
            {
                end = self.next_at(end);
            }
            for _ in 1..count.min(self.text.graphemes(true).count().saturating_add(1)) {
                end = self.word_end(end, big);
            }
            return Some(self.cursor..self.next_at(end));
        }
        let end = self.motion_position(motion, count)?;
        let inclusive = match motion {
            Motion::WordEnd
            | Motion::BigWordEnd
            | Motion::WordEndBackward
            | Motion::BigWordEndBackward
            | Motion::MatchingBracket
            | Motion::Find {
                backward: false, ..
            } => true,
            Motion::RepeatFind(reverse) => {
                matches!(self.last_find, Some(Motion::Find { backward, .. }) if !(backward ^ reverse))
            }
            _ => false,
        };
        let start = self.cursor.min(end);
        let end = if inclusive {
            self.next_at(self.cursor.max(end))
        } else {
            self.cursor.max(end)
        };
        Some(start..end)
    }
}
