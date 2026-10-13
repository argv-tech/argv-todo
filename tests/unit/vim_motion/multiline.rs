use super::*;
use crate::vim_motion::{InputTarget, VimManager};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn keys(editor: &mut Editor, vim: &mut VimManager, text: &str) {
    for character in text.chars() {
        let event = match character {
            '\u{1b}' => KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            '\u{12}' => KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL),
            '\n' => KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            character => KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE),
        };
        if let Some(action) = vim.handle(event) {
            editor.apply(action);
            vim.set_mode(editor.mode());
        }
    }
}

fn normal(text: &str) -> (Editor, VimManager) {
    let mut editor = Editor::default();
    editor.reset_multiline(text.into());
    let mut vim = VimManager::default();
    vim.begin_input(InputTarget::Note);
    keys(&mut editor, &mut vim, "\u{1b}gg");
    (editor, vim)
}

#[test]
fn multiline_paste_normalizes_newlines_and_keeps_undo_and_graphemes() {
    let mut editor = Editor::default();
    editor.reset_multiline("  base".into());
    editor.insert("\r\n界é👩‍💻\r\n\r  tail\n\t");
    assert_eq!(editor.text(), "  base\n界é👩‍💻\n\n  tail\n ");
    editor.apply(VimAction::Input(EditAction::Normal));
    editor.apply(VimAction::Input(EditAction::Undo(1)));
    assert_eq!(editor.text(), "  base");
    editor.apply(VimAction::Input(EditAction::Redo(1)));
    assert_eq!(editor.text(), "  base\n界é👩‍💻\n\n  tail\n ");
    editor.apply(VimAction::Move(Motion::DocumentStart, 1));
    editor.apply(VimAction::Move(Motion::Down, 1));
    editor.apply(VimAction::Move(Motion::Right, 2));
    editor.apply(VimAction::Input(EditAction::Insert(
        InsertPosition::After,
        1,
    )));
    editor.apply(VimAction::Backspace);
    assert_eq!(editor.text(), "  base\n界é\n\n  tail\n ");
    editor.reset("Title".into());
    editor.insert("\r\nnext");
    assert_eq!(editor.text(), "Title  next");
}

#[test]
fn vertical_motions_keep_display_column_across_short_unicode_and_empty_lines() {
    let mut editor = Editor::default();
    editor.reset_multiline("abcde\nx\n界é👩‍💻zz\n".into());
    editor.apply(VimAction::Move(Motion::DocumentStart, 1));
    editor.apply(VimAction::Move(Motion::Right, 3));
    editor.apply(VimAction::Move(Motion::Down, 1));
    assert_eq!(
        &editor.text()[editor.line_start(editor.cursor)..editor.cursor],
        "x"
    );
    editor.apply(VimAction::Move(Motion::Down, 1));
    assert_eq!(
        &editor.text()[editor.line_start(editor.cursor)..editor.cursor],
        "界é"
    );
    editor.apply(VimAction::Move(Motion::Down, 1));
    assert_eq!(editor.cursor, editor.text().len());
    editor.apply(VimAction::Input(EditAction::Normal));
    assert_eq!(editor.cursor, editor.text().len());
    editor.apply(VimAction::Move(Motion::Up, 1));
    assert_eq!(
        &editor.text()[editor.line_start(editor.cursor)..editor.cursor],
        "界é"
    );
    editor.apply(VimAction::Move(Motion::DocumentStart, 1));
    editor.apply(VimAction::Move(Motion::Down, 9999));
    assert_eq!(editor.cursor, editor.text().len());
    editor.apply(VimAction::Move(Motion::Up, 9999));
    assert_eq!(editor.cursor, 0);
}

#[test]
fn vim_line_changes_selections_and_yanks_respect_note_lines() {
    let (mut editor, mut vim) = normal("one\n  two\nthree");
    keys(&mut editor, &mut vim, "jccNew\u{1b}");
    assert_eq!(editor.text(), "one\nNew\nthree");
    keys(&mut editor, &mut vim, "ugg2dd");
    assert_eq!(editor.text(), "three");
    keys(&mut editor, &mut vim, "uggjVjyggp");
    assert_eq!(editor.text(), "one\n  two\nthree\n  two\nthree");
    let (mut editor, mut vim) = normal("one\n\nthree");
    keys(&mut editor, &mut vim, "jV");
    assert_eq!(&editor.text()[editor.selection().unwrap()], "\n");
    keys(&mut editor, &mut vim, "yp");
    assert_eq!(editor.text(), "one\n\n\nthree");
    assert_eq!(editor.cursor, 5);
}

#[test]
fn line_motions_and_find_stay_on_the_current_line() {
    let (mut editor, mut vim) = normal("one\n  two\nthree\n");
    keys(&mut editor, &mut vim, "j^A!\u{1b}");
    assert_eq!(editor.text(), "one\n  two!\nthree\n");
    keys(&mut editor, &mut vim, "0fh");
    assert_eq!(editor.cursor, 4);
    keys(&mut editor, &mut vim, "Gfh");
    assert_eq!(editor.cursor, editor.text().len());
    keys(&mut editor, &mut vim, "ggl");
    assert_eq!(editor.cursor, 1);
    keys(&mut editor, &mut vim, "$l");
    assert_eq!(editor.cursor, 2);
}

#[test]
fn multiline_viewport_keeps_cursor_visible_after_resize_including_trailing_blank_lines() {
    let mut editor = Editor::default();
    editor.reset_multiline("first\n\n界é👩‍💻tail\nlast\n".into());
    for (width, height) in [(20, 8), (4, 2), (1, 1), (0, 0)] {
        let (rows, column, row) = editor.multiline_viewport(width, height);
        if width == 0 || height == 0 {
            assert!(rows.is_empty());
            continue;
        }
        assert!(column < width && row < height);
        assert_eq!(&editor.text()[rows[usize::from(row)].clone()], "");
    }
    editor.apply(VimAction::Move(Motion::Up, 2));
    editor.apply(VimAction::Move(Motion::LineEnd, 1));
    let (rows, column, row) = editor.multiline_viewport(4, 2);
    assert_eq!(&editor.text()[rows[usize::from(row)].clone()], "ail");
    assert_eq!(column, 3);
}
