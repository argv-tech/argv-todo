use super::*;
fn key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

#[test]
fn tab_switches_panes_only_on_press_and_keeps_fields_isolated() {
    let mut vim = VimManager::default();
    for code in [KeyCode::Tab, KeyCode::BackTab] {
        vim.handle(key('p'));
        assert_eq!(
            vim.handle(KeyEvent::new(code, KeyModifiers::NONE)),
            Some(if code == KeyCode::BackTab {
                VimAction::SwitchPaneBackward
            } else {
                VimAction::SwitchPane
            })
        );
        assert!(vim.pending_label().is_empty());
        for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
            let mut event = KeyEvent::new(code, KeyModifiers::NONE);
            event.kind = kind;
            assert_eq!(vim.handle(event), None);
        }
    }
    for target in [
        InputTarget::Task,
        InputTarget::Search,
        InputTarget::DatabasePath,
    ] {
        vim.begin_input(target);
        for mode in [
            VimMode::Insert,
            VimMode::Normal,
            VimMode::Visual,
            VimMode::VisualLine,
            VimMode::Replace,
        ] {
            vim.set_mode(mode);
            assert_eq!(
                vim.handle(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
                None
            );
            assert_eq!(vim.input(), Some(target));
        }
    }
}

#[test]
fn shifted_task_movement_keeps_counts_key_kinds_and_editor_input_distinct() {
    use crate::db::TaskMove;
    let mut vim = VimManager::default();
    for (letter, movement) in [
        ('H', TaskMove::Outdent),
        ('J', TaskMove::Down),
        ('K', TaskMove::Up),
        ('L', TaskMove::Indent),
    ] {
        for code in [letter, letter.to_ascii_lowercase()] {
            vim.handle(key('3'));
            let mut event = KeyEvent::new(KeyCode::Char(code), KeyModifiers::SHIFT);
            assert_eq!(vim.handle(event), Some(VimAction::MoveTask(movement, 3)));
            event.kind = KeyEventKind::Repeat;
            assert_eq!(vim.handle(event), Some(VimAction::MoveTask(movement, 1)));
            event.kind = KeyEventKind::Release;
            assert_eq!(vim.handle(event), None);
        }
        assert_eq!(
            vim.handle(key(letter)),
            Some(VimAction::MoveTask(movement, 1))
        );
        vim.begin_input(InputTarget::Task);
        assert_eq!(vim.handle(key(letter)), Some(VimAction::Insert(letter)));
        vim.end_input();
    }
}

#[test]
fn motions_counts_and_sequences() {
    let mut vim = VimManager::default();
    for (c, motion) in [
        ('h', Motion::Left),
        ('j', Motion::Down),
        ('k', Motion::Up),
        ('l', Motion::Right),
    ] {
        assert_eq!(vim.handle(key(c)), Some(VimAction::Move(motion, 1)));
    }
    assert_eq!(vim.handle(key('3')), None);
    assert_eq!(vim.handle(key('j')), Some(VimAction::Move(Motion::Down, 3)));
    assert_eq!(vim.handle(key('2')), None);
    assert_eq!(vim.handle(key('d')), None);
    assert_eq!(vim.pending_label(), "2d");
    assert_eq!(vim.handle(key('d')), Some(VimAction::Delete(2)));
    assert_eq!(vim.handle(key('g')), None);
    assert_eq!(vim.handle(key('g')), Some(VimAction::FileStart));
}

#[test]
fn insert_mode_treats_commands_as_text_and_ignores_release() {
    let mut vim = VimManager::default();
    vim.begin_input(InputTarget::Task);
    for c in ['h', 'j', 'k', 'l', 'q', 't', 'p', 'i', 'a', 'o', 'O', 'é'] {
        assert_eq!(vim.handle(key(c)), Some(VimAction::Insert(c)));
    }
    let mut release = key('q');
    release.kind = KeyEventKind::Release;
    assert_eq!(vim.handle(release), None);
    assert_eq!(
        vim.handle(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        Some(VimAction::Input(EditAction::Normal))
    );
}

#[test]
fn creation_keys_distinguish_children_and_siblings() {
    let mut vim = VimManager::default();
    for (letter, action) in [
        ('i', VimAction::AddChild),
        ('a', VimAction::AddChild),
        ('o', VimAction::AddBelow),
        ('O', VimAction::AddAbove),
    ] {
        assert_eq!(vim.handle(key(letter)), Some(action));
    }
    assert_eq!(
        vim.handle(KeyEvent::new(KeyCode::Char('O'), KeyModifiers::SHIFT)),
        Some(VimAction::AddAbove)
    );
}

#[test]
fn priority_keys_preserve_vim_counts_and_title_input() {
    let mut vim = VimManager::default();
    assert_eq!(vim.handle(key('t')), Some(VimAction::CyclePriority));
    for (letter, priority) in [
        ('h', Priority::High),
        ('m', Priority::Mid),
        ('l', Priority::Low),
    ] {
        assert_eq!(vim.handle(key('p')), None);
        assert_eq!(vim.pending_label(), "p");
        assert_eq!(vim.handle(key(letter)), Some(VimAction::Priority(priority)));
    }
    vim.handle(key('p'));
    assert_eq!(vim.handle(key('9')), None);
    assert_eq!(vim.handle(key('j')), Some(VimAction::Move(Motion::Down, 1)));
    vim.handle(key('p'));
    vim.handle(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert_eq!(vim.handle(key('h')), Some(VimAction::Move(Motion::Left, 1)));
    vim.begin_input(InputTarget::Task);
    assert_eq!(vim.handle(key('t')), Some(VimAction::Insert('t')));
    assert_eq!(
        vim.handle(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL)),
        Some(VimAction::CyclePriority)
    );
    vim.begin_input(InputTarget::Search);
    assert_eq!(
        vim.handle(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL)),
        None
    );
}

#[test]
fn escape_clears_pending_delete() {
    let mut vim = VimManager::default();
    vim.handle(key('d'));
    vim.handle(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert_eq!(vim.handle(key('d')), None);
    assert_eq!(vim.handle(key('j')), Some(VimAction::Move(Motion::Down, 1)));
    assert_eq!(vim.handle(key('d')), None);
}

#[test]
fn input_sequences_track_counts_and_escape_without_losing_focus() {
    let mut vim = VimManager::default();
    vim.begin_input(InputTarget::Task);
    vim.set_mode(VimMode::Normal);
    for c in "2d3i".chars() {
        assert_eq!(vim.handle(key(c)), None);
    }
    assert_eq!(vim.pending_label(), "2d3i");
    assert_eq!(
        vim.handle(key('w')),
        Some(VimAction::Input(EditAction::Operate(
            Operator::Delete,
            EditTarget::Object(TextObject::Word(false), false),
            6
        )))
    );
    for c in "3d2".chars() {
        vim.handle(key(c));
    }
    assert_eq!(vim.pending_label(), "3d2");
    assert_eq!(
        vim.handle(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        Some(VimAction::Input(EditAction::Normal))
    );
    assert!(vim.pending_label().is_empty());
    assert_eq!(vim.input(), Some(InputTarget::Task));
    assert_eq!(
        vim.handle(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        Some(VimAction::Cancel)
    );

    for c in "999999999999999999999".chars() {
        vim.handle(key(c));
    }
    assert_eq!(
        vim.handle(key('x')),
        Some(VimAction::Input(EditAction::Operate(
            Operator::Delete,
            EditTarget::Motion(Motion::Right),
            9999
        )))
    );
    vim.handle(key('d'));
    assert_eq!(vim.handle(key('?')), None);
    assert!(vim.pending_label().is_empty());
    assert_eq!(vim.handle(key('q')), None);
}

#[test]
fn input_modes_ignore_releases_and_route_repeat_submit_and_replacement_digits() {
    let mut vim = VimManager::default();
    for target in [
        InputTarget::Task,
        InputTarget::Search,
        InputTarget::DatabasePath,
    ] {
        vim.begin_input(target);
        for mode in [
            VimMode::Normal,
            VimMode::Insert,
            VimMode::Visual,
            VimMode::VisualLine,
            VimMode::Replace,
        ] {
            vim.set_mode(mode);
            let mut release = key('d');
            release.kind = KeyEventKind::Release;
            assert_eq!(vim.handle(release), None);
            assert_eq!(
                vim.handle(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
                Some(VimAction::Submit)
            );
            assert_eq!(vim.input(), Some(target));
        }
        vim.set_mode(VimMode::Normal);
        vim.handle(key('r'));
        assert_eq!(
            vim.handle(key('2')),
            Some(VimAction::Input(EditAction::Replace('2', 1)))
        );
        let mut repeat = key('l');
        repeat.kind = KeyEventKind::Repeat;
        assert_eq!(vim.handle(repeat), Some(VimAction::Move(Motion::Right, 1)));
    }
}
