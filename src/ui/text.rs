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
