use super::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, VimAction, VimManager};

impl VimManager {
    pub(crate) fn handle_help(&mut self, key: KeyEvent) -> Option<VimAction> {
        if key.kind == KeyEventKind::Release {
            return None;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            self.reset();
            return match key.code {
                KeyCode::Char('c') => Some(VimAction::Quit),
                KeyCode::Char('[') => Some(VimAction::Cancel),
                _ => None,
            };
        }
        if key
            .modifiers
            .intersects(KeyModifiers::ALT | KeyModifiers::SUPER)
        {
            self.reset();
            return None;
        }
        let action = match key.code {
            KeyCode::Esc | KeyCode::Char('?') => VimAction::Cancel,
            KeyCode::PageDown => VimAction::PageDown,
            KeyCode::PageUp => VimAction::PageUp,
            KeyCode::Tab | KeyCode::BackTab => return self.handle_tasks(key),
            KeyCode::Char('0'..='9' | 'g' | 'G' | 'h' | 'j' | 'k' | 'l' | 'q')
            | KeyCode::Up
            | KeyCode::Down
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::Home
            | KeyCode::End => {
                return self.handle_tasks(key);
            }
            _ => {
                self.reset();
                return None;
            }
        };
        self.reset();
        Some(action)
    }
}
