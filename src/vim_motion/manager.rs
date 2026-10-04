use crate::db::Priority;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::{
    vim_action::{Motion, VimAction},
    vim_mode::VimMode,
};

/// Routes all keyboard input into actions before it reaches the app.
#[derive(Default)]
pub(crate) struct VimManager {
    mode: VimMode,
    count: usize,
    pending: Option<char>,
}

impl VimManager {
    pub(crate) fn mode(&self) -> VimMode {
        self.mode
    }

    pub(crate) fn set_mode(&mut self, mode: VimMode) {
        self.mode = mode;
        self.reset();
    }

    fn reset(&mut self) {
        self.count = 0;
        self.pending = None;
    }

    pub(crate) fn pending_label(&self) -> String {
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

    pub(crate) fn handle(&mut self, key: KeyEvent) -> Option<VimAction> {
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
            KeyCode::Char('i' | 'a') => VimAction::AddChild,
            KeyCode::Char('o') => VimAction::AddBelow,
            KeyCode::Char('O') => VimAction::AddAbove,
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
#[path = "../../tests/unit/vim_motion/manager.rs"]
mod tests;
