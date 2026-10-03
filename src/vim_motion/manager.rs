use crate::db::Priority;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::{
    vim_action::{Motion, VimAction},
    vim_mode::VimMode,
};

/// Routes all keyboard input into actions before it reaches the app.
#[derive(Default)]
pub struct VimManager {
    mode: VimMode,
    count: usize,
    pending: Option<char>,
}

impl VimManager {
    pub fn mode(&self) -> VimMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: VimMode) {
        self.mode = mode;
        self.reset();
    }

    fn reset(&mut self) {
        self.count = 0;
        self.pending = None;
    }

    pub fn pending_label(&self) -> String {
        let count = if self.count == 0 {
            String::new()
        } else {
            self.count.to_string()
        };
        format!(
            "{count}{}",
            self.pending.map_or(String::new(), |c| c.to_string())
        )
    }

    pub fn handle(&mut self, key: KeyEvent) -> Option<VimAction> {
        if key.kind == KeyEventKind::Release {
            return None;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            let action = match key.code {
                KeyCode::Char('c') => Some(VimAction::Quit),
                KeyCode::Char('w') if self.mode != VimMode::Normal => Some(VimAction::DeleteWord),
                KeyCode::Char('u') if self.mode != VimMode::Normal => Some(VimAction::Clear),
                KeyCode::Right if self.mode != VimMode::Normal => {
                    Some(VimAction::Move(Motion::WordForward, 1))
                }
                KeyCode::Left if self.mode != VimMode::Normal => {
                    Some(VimAction::Move(Motion::WordBackward, 1))
                }
                KeyCode::Char('r') if self.mode == VimMode::Normal => Some(VimAction::Refresh),
                KeyCode::Char('p') if self.mode != VimMode::Search => {
                    Some(VimAction::CyclePriority)
                }
                _ => None,
            };
            self.reset();
            return action;
        }
        if key
            .modifiers
            .intersects(KeyModifiers::ALT | KeyModifiers::SUPER)
        {
            return None;
        }
        if key.code == KeyCode::Esc {
            self.reset();
            return Some(VimAction::Cancel);
        }
        if self.mode != VimMode::Normal {
            return match key.code {
                KeyCode::Char(c) => Some(VimAction::Insert(c)),
                KeyCode::Enter => Some(VimAction::Submit),
                KeyCode::Backspace => Some(VimAction::Backspace),
                KeyCode::Delete => Some(VimAction::DeleteChar),
                KeyCode::Left => Some(VimAction::Move(Motion::Left, 1)),
                KeyCode::Right => Some(VimAction::Move(Motion::Right, 1)),
                KeyCode::Home => Some(VimAction::Move(Motion::LineStart, 1)),
                KeyCode::End => Some(VimAction::Move(Motion::LineEnd, 1)),
                _ => None,
            };
        }
        if let KeyCode::Char(c @ '0'..='9') = key.code
            && (c != '0' || self.count > 0)
            && self.pending.is_none()
        {
            self.count = (self.count * 10 + c.to_digit(10).unwrap() as usize).min(9999);
            return None;
        }
        let count = self.count.max(1);
        if let Some(prefix) = self.pending.take() {
            if prefix == 'p' {
                let action = match key.code {
                    KeyCode::Char('h') => Some(VimAction::Priority(Priority::High)),
                    KeyCode::Char('m') => Some(VimAction::Priority(Priority::Mid)),
                    KeyCode::Char('l') => Some(VimAction::Priority(Priority::Low)),
                    _ => None,
                };
                self.reset();
                return action;
            }
            let action = match (prefix, key.code) {
                ('g', KeyCode::Char('g')) => Some(VimAction::FileStart),
                ('d', KeyCode::Char('d')) => Some(VimAction::Delete(count)),
                ('c', KeyCode::Char('c')) => Some(VimAction::Edit),
                _ => None,
            };
            if action.is_some() {
                self.reset();
                return action;
            }
        }
        let action = match key.code {
            KeyCode::Char(c @ ('g' | 'd' | 'c' | 'p')) => {
                self.pending = Some(c);
                return None;
            }
            KeyCode::Char('h') | KeyCode::Left => VimAction::Move(Motion::Left, count),
            KeyCode::Char('j') | KeyCode::Down => VimAction::Move(Motion::Down, count),
            KeyCode::Char('k') | KeyCode::Up => VimAction::Move(Motion::Up, count),
            KeyCode::Char('l') | KeyCode::Right => VimAction::Move(Motion::Right, count),
            KeyCode::Char('e') => VimAction::Edit,
            KeyCode::Char('t') => VimAction::CyclePriority,
            KeyCode::Char('0') | KeyCode::Home => VimAction::FileStart,
            KeyCode::Char('G') | KeyCode::End => VimAction::FileEnd,
            KeyCode::Char('i' | 'a' | 'o') => VimAction::Add,
            KeyCode::Char(' ' | 'x') | KeyCode::Enter => VimAction::Toggle,
            KeyCode::Char('u') => VimAction::Undo,
            KeyCode::Char('/') => VimAction::Search,
            KeyCode::Char('?') => VimAction::Help,
            KeyCode::Char('q') => VimAction::Quit,
            _ => {
                self.reset();
                return None;
            }
        };
        self.reset();
        Some(action)
    }
}

#[cfg(test)]
mod tests {
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
        for c in ['h', 'j', 'k', 'l', 'q', 't', 'p', 'é'] {
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
}
