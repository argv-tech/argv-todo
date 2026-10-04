use super::{document, wrap};
use unicode_width::UnicodeWidthStr;

#[test]
fn help_content_fits_each_available_width() {
    assert!(document(0).lines.is_empty());
    for width in [1, 2, 16, 29, 51, 52, 74] {
        let lines = document(width).lines;
        assert!(!lines.is_empty());
        assert!(lines.iter().all(|line| line.width() <= usize::from(width)));
        assert!(!lines.last().unwrap().to_string().is_empty());
        if width >= 16 {
            assert!(lines.last().unwrap().to_string().ends_with("Visual"));
        }
    }
}

#[test]
fn wrapping_keeps_unicode_graphemes_and_splits_long_words() {
    let wrapped = wrap("界é👩‍💻 abcdefgh", 4);
    assert_eq!(wrapped, ["界é", "👩‍💻", "abcd", "efgh"]);
    assert!(wrapped.iter().all(|line| line.width() <= 4));
    assert_eq!(wrap("two words", 9), ["two words"]);
    assert_eq!(wrap("two words", 8), ["two", "words"]);
}
