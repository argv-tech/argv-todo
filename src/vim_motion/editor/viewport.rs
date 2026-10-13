use super::Editor;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

impl Editor {
    /// Logical lines scroll vertically; the focused line also scrolls horizontally.
    pub(crate) fn multiline_viewport(
        &self,
        width: u16,
        height: u16,
    ) -> (Vec<Range<usize>>, u16, u16) {
        if width == 0 || height == 0 {
            return (Vec::new(), 0, 0);
        }
        let mut start = 0;
        let mut lines: Vec<_> = self
            .text
            .split('\n')
            .map(|line| {
                let range = start..start + line.len();
                start += line.len() + 1;
                range
            })
            .collect();
        let cursor_row = lines
            .iter()
            .position(|line| line.contains(&self.cursor) || line.end == self.cursor)
            .unwrap_or(0);
        let focused = &mut lines[cursor_row];
        while self.text[focused.start..self.cursor].width() >= usize::from(width) {
            let Some(grapheme) = self.text[focused.start..self.cursor].graphemes(true).next()
            else {
                break;
            };
            focused.start += grapheme.len();
        }
        let column = self.text[focused.start..self.cursor].width() as u16;
        let height = usize::from(height);
        let offset = (cursor_row + 1)
            .saturating_sub(height)
            .min(lines.len().saturating_sub(height));
        (
            lines.into_iter().skip(offset).take(height).collect(),
            column,
            (cursor_row - offset) as u16,
        )
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
