use super::Editor;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

impl Editor {
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
