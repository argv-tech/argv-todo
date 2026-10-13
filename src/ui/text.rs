use std::borrow::Cow;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Fit a single-line label without cutting a grapheme or hiding truncation.
pub(super) fn truncate(text: &str, width: usize) -> Cow<'_, str> {
    if text.width() <= width {
        return Cow::Borrowed(text);
    }
    if width == 0 {
        return Cow::Borrowed("");
    }
    let mut used = 0;
    let mut end = 0;
    for (byte, grapheme) in text.grapheme_indices(true) {
        let next = used + grapheme.width();
        if next > width - 1 {
            break;
        }
        used = next;
        end = byte + grapheme.len();
    }
    Cow::Owned(format!("{}…", &text[..end]))
}

/// Wrap at words and grapheme boundaries so previews and help share row geometry.
pub(super) fn wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut used = 0;
    for word in text.split_whitespace() {
        if !line.is_empty() && used + 1 + word.width() > width {
            lines.push(std::mem::take(&mut line));
            used = 0;
        }
        if !line.is_empty() {
            line.push(' ');
            used += 1;
        }
        for grapheme in word.graphemes(true) {
            if used + grapheme.width() > width && !line.is_empty() {
                lines.push(std::mem::take(&mut line));
                used = 0;
            }
            if grapheme.width() > width {
                line.push('…');
                used += 1;
            } else {
                line.push_str(grapheme);
                used += grapheme.width();
            }
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}
