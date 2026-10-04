use super::*;
#[test]
fn edits_unicode_at_cursor_and_deletes_whole_graphemes() {
    let mut editor = Editor::new("café 👩‍💻".into());
    editor.apply(VimAction::Backspace);
    assert_eq!(editor.text(), "café ");
    editor.apply(VimAction::Move(Motion::Left, 1));
    editor.insert("!");
    assert_eq!(editor.text(), "café! ");
    editor.apply(VimAction::Move(Motion::LineStart, 1));
    editor.apply(VimAction::DeleteChar);
    assert_eq!(editor.text(), "afé! ");
}

#[test]
fn word_delete_paste_and_horizontal_scroll() {
    let mut editor = Editor::new("first second".into());
    editor.apply(VimAction::DeleteWord);
    assert_eq!(editor.text(), "first ");
    editor.insert("界\nend");
    assert_eq!(editor.text(), "first 界 end");
    let (text, col) = editor.viewport(5);
    assert_eq!(text, " end");
    assert_eq!(col, 4);
}

#[test]
fn deletion_that_joins_a_flag_keeps_cursor_on_a_boundary() {
    let mut editor = Editor::new("🇦x🇧".into());
    editor.apply(VimAction::Move(Motion::Left, 1));
    editor.apply(VimAction::Backspace);
    assert_eq!(editor.text(), "🇦🇧");
    editor.apply(VimAction::Backspace);
    assert_eq!(editor.text(), "");
}
