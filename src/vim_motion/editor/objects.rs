use super::motions::WordClass;
use super::{Editor, TextObject};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

impl Editor {
    pub(super) fn object_range(
        &self,
        object: TextObject,
        around: bool,
        count: usize,
    ) -> Option<Range<usize>> {
        match object {
            TextObject::Word(big) => self.word_range(big, around, count),
            TextObject::Delimited(delimiter) => self.delimited_range(delimiter, around, count),
        }
    }

    fn word_range(&self, big: bool, around: bool, count: usize) -> Option<Range<usize>> {
        if self.text.is_empty() {
            return None;
        }
        let cursor = self.cursor.min(self.previous_at(self.text.len()));
        let class = self.class_at(cursor, big);
        let mut start = cursor;
        while start > 0 && self.class_at(self.previous_at(start), big) == class {
            start = self.previous_at(start);
        }
        let mut end = self.next_at(cursor);
        while end < self.text.len() && self.class_at(end, big) == class {
            end = self.next_at(end);
        }
        for _ in 1..count.min(self.text.graphemes(true).count().saturating_add(1)) {
            if end == self.text.len() {
                break;
            }
            if around {
                while end < self.text.len() && self.class_at(end, big) == WordClass::Space {
                    end = self.next_at(end);
                }
            }
            if end == self.text.len() {
                break;
            }
            let class = self.class_at(end, big);
            while end < self.text.len() && self.class_at(end, big) == class {
                end = self.next_at(end);
            }
        }
        if around {
            if class == WordClass::Space {
                if end < self.text.len() {
                    let next_class = self.class_at(end, big);
                    while end < self.text.len() && self.class_at(end, big) == next_class {
                        end = self.next_at(end);
                    }
                }
            } else {
                // Prefer trailing whitespace; at the end use leading whitespace.
                let word_end = end;
                while end < self.text.len() && self.class_at(end, big) == WordClass::Space {
                    end = self.next_at(end);
                }
                if end == word_end {
                    while start > 0
                        && self.class_at(self.previous_at(start), big) == WordClass::Space
                    {
                        start = self.previous_at(start);
                    }
                }
            }
        }
        Some(start..end)
    }

    fn delimited_range(&self, delimiter: char, around: bool, count: usize) -> Option<Range<usize>> {
        let (open, close) = pair(delimiter)?;
        let mut stack = Vec::new();
        let mut pairs = Vec::new();
        let mut escaped = false;
        for (position, grapheme) in self.text.grapheme_indices(true) {
            let character = grapheme.chars().next()?;
            if open == close && escaped {
                escaped = false;
                continue;
            }
            if open == close && character == '\\' {
                escaped = true;
                continue;
            }
            if character == close && !stack.is_empty() {
                let start = stack.pop()?;
                pairs.push(start..position);
            } else if character == open {
                stack.push(position);
            }
        }
        let mut enclosing: Vec<_> = pairs
            .iter()
            .filter(|range| range.start <= self.cursor && self.cursor <= range.end)
            .cloned()
            .collect();
        enclosing.sort_by_key(|range| range.end - range.start);
        // Quote objects also find the next quoted string on the same line.
        let range = enclosing
            .get(count.saturating_sub(1))
            .cloned()
            .or_else(|| {
                if open == close {
                    pairs
                        .iter()
                        .filter(|range| range.start > self.cursor)
                        .min_by_key(|range| range.start)
                        .cloned()
                } else {
                    None
                }
            })?;
        Some(if around {
            range.start..self.next_at(range.end)
        } else {
            self.next_at(range.start)..range.end
        })
    }

    pub(super) fn matching_bracket(&self) -> Option<usize> {
        let (position, character) =
            self.text[self.cursor..]
                .grapheme_indices(true)
                .find_map(|(i, g)| {
                    let character = g.chars().next()?;
                    matches!(character, '(' | ')' | '[' | ']' | '{' | '}')
                        .then_some((self.cursor + i, character))
                })?;
        let (open, close) = pair(character)?;
        let backward = character == close;
        let mut depth = 1usize;
        if backward {
            for (i, g) in self.text[..position].grapheme_indices(true).rev() {
                let c = g.chars().next()?;
                if c == close {
                    depth += 1;
                }
                if c == open {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
            }
        } else {
            let start = self.next_at(position);
            for (i, g) in self.text[start..].grapheme_indices(true) {
                let c = g.chars().next()?;
                if c == open {
                    depth += 1;
                }
                if c == close {
                    depth -= 1;
                    if depth == 0 {
                        return Some(start + i);
                    }
                }
            }
        }
        None
    }
}

fn pair(delimiter: char) -> Option<(char, char)> {
    Some(match delimiter {
        '(' | ')' => ('(', ')'),
        '{' | '}' => ('{', '}'),
        '[' | ']' => ('[', ']'),
        '<' | '>' => ('<', '>'),
        c @ ('\'' | '"' | '`') => (c, c),
        _ => return None,
    })
}
