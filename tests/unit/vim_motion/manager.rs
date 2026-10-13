use super::*;

#[test]
fn archive_restore_and_clear_use_separate_press_only_keys() {
    let mut vim = VimManager::default();
    for (code, modifiers, action) in [
        (
            KeyCode::Char('r'),
            KeyModifiers::NONE,
            VimAction::RestoreArchived,
        ),
        (
            KeyCode::Char('D'),
            KeyModifiers::NONE,
            VimAction::ClearArchived,
        ),
        (
            KeyCode::Char('D'),
            KeyModifiers::SHIFT,
            VimAction::ClearArchived,
        ),
        (
            KeyCode::Char('d'),
            KeyModifiers::SHIFT,
            VimAction::ClearArchived,
        ),
        (KeyCode::Enter, KeyModifiers::NONE, VimAction::Submit),
    ] {
        vim.handle(key('3'));
        assert_eq!(
            vim.handle_archive(KeyEvent::new(code, modifiers)),
            Some(action)
        );
        assert!(vim.pending_label().is_empty());
        for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
            assert_eq!(
                vim.handle_archive(KeyEvent::new_with_kind(code, modifiers, kind)),
                None
            );
        }
        assert_eq!(
            vim.handle(KeyEvent::new(code, modifiers)),
            if code == KeyCode::Enter {
                Some(VimAction::ToggleCollapse)
            } else {
                None
            }
        );
    }
    assert_eq!(
        vim.handle_archive(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL)),
        Some(VimAction::Refresh)
    );
    assert_eq!(
        vim.handle_archive(KeyEvent::new(KeyCode::Char('D'), KeyModifiers::ALT)),
        None
    );
    assert_eq!(vim.handle_archive(key('d')), None);
    assert_eq!(vim.handle_archive(key('d')), Some(VimAction::Delete(1)));
    assert_eq!(
        vim.handle_archive(key('j')),
        Some(VimAction::Move(Motion::Down, 1))
    );
    for target in [
        InputTarget::Task,
        InputTarget::Note,
        InputTarget::Search,
        InputTarget::DatabasePath,
        InputTarget::ProjectName,
    ] {
        vim.begin_input(target);
        for letter in ['r', 'D'] {
            assert_eq!(
                vim.handle_archive(key(letter)),
                Some(VimAction::Insert(letter))
            );
        }
        vim.end_input();
    }
}

#[test]
fn archive_shortcut_is_press_only_and_keeps_editor_append_and_modal_focus() {
    let mut vim = VimManager::default();
    for key in [
        KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('a'), KeyModifiers::SHIFT),
    ] {
        assert_eq!(vim.handle(key), Some(VimAction::ShowDeleted));
        for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
            assert_eq!(
                vim.handle(KeyEvent::new_with_kind(key.code, key.modifiers, kind)),
                None
            );
        }
    }
    assert_eq!(
        vim.handle(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::ALT)),
        None
    );
    assert_eq!(
        vim.handle_project_picker(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE)),
        None
    );
    assert_eq!(
        vim.handle_help(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE)),
        None
    );
    for target in [
        InputTarget::Task,
        InputTarget::Note,
        InputTarget::Search,
        InputTarget::DatabasePath,
        InputTarget::ProjectName,
    ] {
        vim.begin_input(target);
        assert_eq!(
            vim.handle(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT)),
            Some(VimAction::Insert('A'))
        );
        vim.set_mode(VimMode::Normal);
        assert!(matches!(
            vim.handle(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT)),
            Some(VimAction::Input(EditAction::Insert(InsertPosition::End, 1)))
        ));
        vim.end_input();
    }
}

#[test]
fn project_commands_and_picker_keep_fields_modifiers_and_key_kinds_isolated() {
    let mut manager = VimManager::default();
    assert_eq!(
        manager.handle(KeyEvent::new(KeyCode::Char('P'), KeyModifiers::SHIFT)),
        Some(VimAction::AddProject)
    );
    assert_eq!(
        manager.handle(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE)),
        Some(VimAction::AssignProject)
    );
    manager.begin_input(InputTarget::ProjectName);
    assert_eq!(
        manager.handle(KeyEvent::new(KeyCode::Char('P'), KeyModifiers::SHIFT)),
        Some(VimAction::Insert('P'))
    );
    assert_eq!(
        manager.handle(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE)),
        Some(VimAction::Insert('m'))
    );
    assert_eq!(
        manager.handle(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL)),
        None
    );
    manager.end_input();
    for kind in [KeyEventKind::Release, KeyEventKind::Repeat] {
        assert_eq!(
            manager.handle_project_picker(KeyEvent::new_with_kind(
                KeyCode::Enter,
                KeyModifiers::NONE,
                kind
            )),
            None
        );
    }
    for code in [
        KeyCode::Char('P'),
        KeyCode::Char('i'),
        KeyCode::Char('x'),
        KeyCode::Char('d'),
    ] {
        assert_eq!(
            manager.handle_project_picker(KeyEvent::new(code, KeyModifiers::NONE)),
            None
        );
    }
    assert_eq!(
        manager.handle_project_picker(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::ALT)),
        None
    );
    assert_eq!(
        manager.handle_project_picker(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)),
        Some(VimAction::Move(Motion::Up, 1))
    );
    assert_eq!(
        manager.handle_project_picker(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        Some(VimAction::Submit)
    );
    assert_eq!(
        manager.handle_project_picker(KeyEvent::new(KeyCode::Char('['), KeyModifiers::CONTROL)),
        Some(VimAction::Cancel)
    );
}

#[test]
fn conversion_and_cleanup_accept_press_only_and_keep_editor_commands() {
    let mut vim = VimManager::default();
    for event in [
        KeyEvent::new(KeyCode::Char('C'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::SHIFT),
        KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT),
    ] {
        let expected = if event.code == KeyCode::Enter {
            VimAction::ClearCompleted
        } else {
            VimAction::ConvertProject
        };
        assert_eq!(vim.handle(event), Some(expected));
        for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
            assert_eq!(
                vim.handle(KeyEvent::new_with_kind(event.code, event.modifiers, kind)),
                None
            );
        }
    }
    for target in [
        InputTarget::Task,
        InputTarget::Note,
        InputTarget::Search,
        InputTarget::ProjectName,
        InputTarget::DatabasePath,
    ] {
        vim.begin_input(target);
        assert_eq!(
            vim.handle(KeyEvent::new(KeyCode::Char('C'), KeyModifiers::SHIFT)),
            Some(VimAction::Insert('C'))
        );
        assert_eq!(
            vim.handle(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)),
            None
        );
        vim.set_mode(VimMode::Normal);
        assert!(matches!(
            vim.handle(KeyEvent::new(KeyCode::Char('C'), KeyModifiers::SHIFT)),
            Some(VimAction::Input(_))
        ));
    }
    vim.end_input();
    assert_eq!(
        vim.handle_help(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)),
        None
    );
    assert_eq!(
        vim.handle_project_picker(KeyEvent::new(KeyCode::Char('C'), KeyModifiers::SHIFT)),
        None
    );
    assert_eq!(
        vim.handle_project_picker(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)),
        None
    );
}
fn key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

#[test]
fn project_tab_mapping_honors_counts_key_kinds_and_existing_commands() {
    let mut vim = VimManager::default();
    for (letter, expected) in [
        ('h', VimAction::ProjectLeft(2)),
        ('l', VimAction::ProjectRight(2)),
    ] {
        vim.handle_projects(key('2'));
        assert_eq!(vim.handle_projects(key(letter)), Some(expected));
        assert!(vim.pending_label().is_empty());
        assert_eq!(
            vim.handle_projects(KeyEvent::new_with_kind(
                KeyCode::Char(letter),
                KeyModifiers::NONE,
                KeyEventKind::Repeat,
            )),
            Some(if letter == 'h' {
                VimAction::ProjectLeft(1)
            } else {
                VimAction::ProjectRight(1)
            })
        );
        assert_eq!(
            vim.handle_projects(KeyEvent::new_with_kind(
                KeyCode::Char(letter),
                KeyModifiers::NONE,
                KeyEventKind::Release,
            )),
            None
        );
        for modifiers in [
            KeyModifiers::CONTROL,
            KeyModifiers::ALT,
            KeyModifiers::SUPER,
        ] {
            assert_eq!(
                vim.handle_projects(KeyEvent::new(KeyCode::Char(letter), modifiers)),
                None
            );
        }
    }
    for (code, motion) in [
        (KeyCode::Left, Motion::Left),
        (KeyCode::Right, Motion::Right),
    ] {
        assert_eq!(
            vim.handle_projects(KeyEvent::new(code, KeyModifiers::NONE)),
            Some(VimAction::Move(motion, 1))
        );
    }
    assert_eq!(
        vim.handle_projects(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::SHIFT)),
        Some(VimAction::MoveTask(crate::db::TaskMove::Outdent, 1))
    );
    assert_eq!(
        vim.handle_projects(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::SHIFT)),
        Some(VimAction::MoveTask(crate::db::TaskMove::Indent, 1))
    );
    for target in [
        InputTarget::Task,
        InputTarget::Note,
        InputTarget::Search,
        InputTarget::DatabasePath,
        InputTarget::ProjectName,
    ] {
        vim.begin_input(target);
        for letter in ['h', 'l'] {
            assert_eq!(
                vim.handle_projects(key(letter)),
                Some(VimAction::Insert(letter))
            );
        }
        vim.set_mode(VimMode::Normal);
        assert_eq!(
            vim.handle_projects(key('h')),
            Some(VimAction::Input(EditAction::Move(Motion::Left, 1)))
        );
        assert_eq!(
            vim.handle_projects(key('l')),
            Some(VimAction::Input(EditAction::Move(Motion::Right, 1)))
        );
        vim.end_input();
    }
}

#[test]
fn return_folds_only_on_press_and_completion_keeps_its_own_keys() {
    let mut vim = VimManager::default();
    vim.handle(key('3'));
    assert_eq!(
        vim.handle(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        Some(VimAction::ToggleCollapse)
    );
    assert!(vim.pending_label().is_empty());
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        let mut key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        key.kind = kind;
        assert_eq!(vim.handle(key), None);
    }
    for c in [' ', 'x'] {
        assert_eq!(vim.handle(key(c)), Some(VimAction::Toggle));
    }
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
        InputTarget::Note,
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
    vim.end_input();
    assert_eq!(vim.handle(key('n')), Some(VimAction::EditNote));
    vim.begin_input(InputTarget::Note);
    assert_eq!(vim.handle(key('n')), Some(VimAction::Insert('n')));
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
