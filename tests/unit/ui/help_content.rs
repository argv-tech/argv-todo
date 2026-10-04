use super::{document, wrap};
use unicode_width::UnicodeWidthStr;

#[test]
fn help_guide_and_content_omit_field_editing_sections() {
    for width in [29, 74] {
        let document = document(width);
        let titles = document
            .sections
            .iter()
            .map(|section| section.title)
            .collect::<Vec<_>>();
        assert_eq!(
            titles,
            [
                "TASKS / Navigation",
                "TASKS / Create & change",
                "APP / Search & settings",
            ]
        );
        let content = document
            .lines
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        for removed in ["FIELDS /", "Visual", "yank", "matching bracket"] {
            assert!(!content.contains(removed), "help still contains {removed}");
        }
    }
}

#[test]
fn help_content_fits_each_available_width() {
    assert!(document(0).lines.is_empty());
    for width in [1, 2, 16, 29, 51, 52, 74] {
        let lines = document(width).lines;
        assert!(!lines.is_empty());
        assert!(lines.iter().all(|line| line.width() <= usize::from(width)));
        assert!(!lines.last().unwrap().to_string().is_empty());
        if width >= 16 {
            assert!(lines.last().unwrap().to_string().ends_with("anywhere"));
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
