use super::*;
fn key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
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
    vim.set_mode(VimMode::Insert);
    for c in ['h', 'j', 'k', 'l', 'q', 't', 'p', 'i', 'a', 'o', 'O', 'é'] {
        assert_eq!(vim.handle(key(c)), Some(VimAction::Insert(c)));
    }
    let mut release = key('q');
    release.kind = KeyEventKind::Release;
    assert_eq!(vim.handle(release), None);
    assert_eq!(
        vim.handle(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        Some(VimAction::Cancel)
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
    vim.set_mode(VimMode::Insert);
    assert_eq!(vim.handle(key('t')), Some(VimAction::Insert('t')));
    assert_eq!(
        vim.handle(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL)),
        Some(VimAction::CyclePriority)
    );
    vim.set_mode(VimMode::Search);
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
