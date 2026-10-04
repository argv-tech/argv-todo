use super::*;
use crate::vim_motion::{InputTarget, VimManager};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn press(editor: &mut Editor, vim: &mut VimManager, character: char) {
    let key = match character {
        '\u{1b}' => KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
        '\u{12}' => KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL),
        '\u{8}' => KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
        c => KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE),
    };
    if let Some(action) = vim.handle(key) {
        editor.apply(action);
        vim.set_mode(editor.mode());
    }
}

fn keys(editor: &mut Editor, vim: &mut VimManager, text: &str) {
    for character in text.chars() {
        press(editor, vim, character);
    }
}

fn normal(text: &str) -> (Editor, VimManager) {
    let mut editor = Editor::new(text.to_owned());
    let mut vim = VimManager::default();
    vim.begin_input(InputTarget::Task);
    press(&mut editor, &mut vim, '\u{1b}');
    (editor, vim)
}

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

#[test]
fn operators_counts_and_text_objects_edit_the_expected_ranges() {
    for (text, commands, expected) in [
        ("one two three four", "03dw", "four"),
        ("one two three four five six seven", "02d3w", "seven"),
        ("one two three", "0d2w", "three"),
        ("one two", "0ldiw", " two"),
        ("one two", "0daw", "two"),
        ("one two three", "0d2iw", "two three"),
        ("one two three", "0d2aw", "three"),
        ("first word", "0wdiw", "first "),
        ("first word", "0wdaw", "first"),
        ("some/path.file end", "0diW", " end"),
        ("some/path.file end", "0diw", "/path.file end"),
        ("a {one {two} end} z", "0ftdi{", "a {one {} end} z"),
        ("a {one {two} end} z", "0ftd2i{", "a {} z"),
        ("a {one {two} end} z", "0ftda}", "a {one  end} z"),
        ("a (one) z", "0fodi)", "a () z"),
        ("a [one] z", "0foda[", "a  z"),
        ("a <one> z", "0fodi>", "a <> z"),
        ("a \"one\" z", "0di\"", "a \"\" z"),
        ("a 'one' z", "0foda'", "a  z"),
        ("a `one` z", "0fodi`", "a `` z"),
        ("a \"one \\\"two\\\"\" z", "0di\"", "a \"\" z"),
        ("whole field", "dd", ""),
        ("whole field", "3dd", ""),
        ("one two", "0cwnew\u{1b}", "new two"),
        ("a two", "0cwnew\u{1b}", "new two"),
        ("one two", "0ecwnew\u{1b}", "onnew two"),
        ("whole field", "ccnew\u{1b}", "new"),
        ("abcde", "03r2", "222de"),
        ("abcde", "r2", "abcd2"),
        ("abcde", "03x", "de"),
        ("abcde", "2X", "abe"),
        ("abcde", "02lD", "ab"),
        ("abcde", "0sX\u{1b}", "Xbcde"),
        ("MiXeD text", "0gUiw", "MIXED text"),
        ("MiXeD text", "0guiw", "mixed text"),
        ("MiXeD text", "0g~iw", "mIxEd text"),
        ("ab,cd,ef", "0dt,", ",cd,ef"),
        ("ab,cd,ef", "0df,", "cd,ef"),
        ("ab,cd,ef", "dF,", "ab,cdf"),
        ("ab,cd,ef", "dT,", "ab,cd,f"),
        ("a{b}c", "0f{d%", "ac"),
    ] {
        let (mut editor, mut vim) = normal(text);
        keys(&mut editor, &mut vim, commands);
        assert_eq!(editor.text(), expected, "{text:?}: {commands:?}");
        assert_eq!(editor.mode(), VimMode::Normal);
    }
}

#[test]
fn visual_selection_yank_put_change_and_swap_work_in_both_directions() {
    let (mut editor, mut vim) = normal("one two");
    keys(&mut editor, &mut vim, "0viw");
    assert_eq!(editor.selection(), Some(0..3));
    keys(&mut editor, &mut vim, "o");
    assert_eq!(editor.cursor, 0);
    assert_eq!(editor.selection(), Some(0..3));
    keys(&mut editor, &mut vim, "y$p");
    assert_eq!(editor.text(), "one twoone");
    keys(&mut editor, &mut vim, "0wviwcX\u{1b}");
    assert_eq!(editor.text(), "one X");
    keys(&mut editor, &mut vim, "V");
    assert_eq!(editor.selection(), Some(0..5));
    keys(&mut editor, &mut vim, "d");
    assert_eq!(editor.text(), "");
    assert_eq!(editor.mode(), VimMode::Normal);

    let (mut editor, mut vim) = normal("abcdef");
    keys(&mut editor, &mut vim, "v2hr2");
    assert_eq!(editor.text(), "abc222");
    keys(&mut editor, &mut vim, "0vlu");
    assert_eq!(editor.mode(), VimMode::Normal);
}

#[test]
fn motions_handle_words_punctuation_columns_and_find_repeats() {
    for (commands, expected) in [
        ("0w", 3),
        ("0W", 8),
        ("0e", 2),
        ("0E", 6),
        ("0wge", 2),
        ("B", 8),
        ("0f,;", 13),
        ("0t,;", 12),
        ("0f,;,", 3),
    ] {
        let (mut editor, mut vim) = normal("one,two three,x,y");
        keys(&mut editor, &mut vim, commands);
        assert_eq!(editor.cursor, expected, "{commands}");
    }
    let (mut editor, mut vim) = normal("界é👩‍💻z");
    for (commands, expected) in [("2|", 0), ("3|", 3), ("4|", 6), ("6|", 17)] {
        keys(&mut editor, &mut vim, commands);
        assert_eq!(editor.cursor, expected);
    }
}

#[test]
fn undo_redo_repeat_and_counted_insert_preserve_transactions() {
    let (mut editor, mut vim) = normal("one two three");
    keys(&mut editor, &mut vim, "0dw.");
    assert_eq!(editor.text(), "three");
    keys(&mut editor, &mut vim, "2u");
    assert_eq!(editor.text(), "one two three");
    keys(&mut editor, &mut vim, "\u{12}\u{12}");
    assert_eq!(editor.text(), "three");
    keys(&mut editor, &mut vim, "0ciwnew\u{1b}");
    assert_eq!(editor.text(), "new");
    keys(&mut editor, &mut vim, "u");
    assert_eq!(editor.text(), "three");
    keys(&mut editor, &mut vim, ".");
    assert_eq!(editor.text(), "new");
    keys(&mut editor, &mut vim, "03iX\u{1b}");
    assert_eq!(editor.text(), "XXXnew");
    keys(&mut editor, &mut vim, "$.");
    assert_eq!(editor.text(), "XXXneXXXw");
    keys(&mut editor, &mut vim, "u");
    assert_eq!(editor.text(), "XXXnew");
    keys(&mut editor, &mut vim, "u");
    assert_eq!(editor.text(), "new");
}

#[test]
fn replace_mode_backspace_restores_text_and_unicode_edits_stay_on_boundaries() {
    let (mut editor, mut vim) = normal("界é👩‍💻 end");
    keys(&mut editor, &mut vim, "0RXY\u{8}\u{1b}");
    assert_eq!(editor.text(), "Xé👩‍💻 end");
    keys(&mut editor, &mut vim, "u");
    assert_eq!(editor.text(), "界é👩‍💻 end");
    keys(&mut editor, &mut vim, "0l2x");
    assert_eq!(editor.text(), "界 end");
    keys(&mut editor, &mut vim, "u0v2ld");
    assert_eq!(editor.text(), " end");
    assert!(editor.text.is_char_boundary(editor.cursor));
}

#[test]
fn failed_commands_empty_fields_and_long_delimiters_are_safe() {
    for text in ["", "a", "🇦x🇧", "界é👩‍💻", "{broken"] {
        for commands in [
            "0di{",
            "0diw",
            "0fz",
            "09r2",
            "0d?",
            "0g?",
            "0v$yP",
            "0Vd",
            "0d0",
            "0ge",
            "0gE",
            "0cwX\u{1b}",
        ] {
            let (mut editor, mut vim) = normal(text);
            keys(&mut editor, &mut vim, commands);
            assert!(
                editor.text.is_char_boundary(editor.cursor),
                "{text}: {commands}"
            );
            assert!(
                editor.cursor == editor.text.len()
                    || editor
                        .text
                        .grapheme_indices(true)
                        .any(|(i, _)| i == editor.cursor)
            );
        }
    }
    let text = format!("{{{}}}", "界".repeat(22000));
    let (mut editor, mut vim) = normal(&text);
    keys(&mut editor, &mut vim, "0ldi{");
    assert_eq!(editor.text(), "{}");
}

#[test]
fn paste_is_an_undoable_edit_and_register_survives_field_changes() {
    let (mut editor, mut vim) = normal("one two");
    keys(&mut editor, &mut vim, "0yiw");
    editor.reset("x".into());
    vim.begin_input(InputTarget::Search);
    keys(&mut editor, &mut vim, "\u{1b}p");
    assert_eq!(editor.text(), "xone");
    keys(&mut editor, &mut vim, "0viw");
    let selection = editor.selection();
    editor.insert("");
    assert_eq!(editor.selection(), selection);
    editor.insert("界\n👩‍💻");
    vim.set_mode(editor.mode());
    assert_eq!(editor.text(), "界 👩‍💻");
    keys(&mut editor, &mut vim, "u");
    assert_eq!(editor.text(), "xone");
}

#[test]
fn visual_changes_repeat_by_selection_length_and_empty_objects_do_not_delete_delimiters() {
    let (mut editor, mut vim) = normal("abcdef");
    keys(&mut editor, &mut vim, "0vld.");
    assert_eq!(editor.text(), "ef");
    keys(&mut editor, &mut vim, "u");
    assert_eq!(editor.text(), "cdef");

    let (mut editor, mut vim) = normal("{} tail");
    keys(&mut editor, &mut vim, "0vi{d");
    // The incomplete operator is cancelled before a different command.
    keys(&mut editor, &mut vim, "\u{1b}0ci{X\u{1b}");
    assert_eq!(editor.text(), "{X} tail");
}
