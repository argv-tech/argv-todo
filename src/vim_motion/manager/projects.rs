use super::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, Motion, VimAction, VimManager};

impl VimManager {
    pub(crate) fn handle_projects(&mut self, key: KeyEvent) -> Option<VimAction> {
        if self.input.is_some() {
            return self.handle(key);
        }
        let action = self.handle(key)?;
        Some(match (key.code, action) {
            (KeyCode::Char('h'), VimAction::Move(Motion::Left, count)) => {
                VimAction::ProjectLeft(count)
            }
            (KeyCode::Char('l'), VimAction::Move(Motion::Right, count)) => {
                VimAction::ProjectRight(count)
            }
            _ => action,
        })
    }

    pub(crate) fn handle_project_picker(&mut self, key: KeyEvent) -> Option<VimAction> {
        if key.kind == KeyEventKind::Release {
            return None;
        }
        self.reset();
        if key
            .modifiers
            .intersects(KeyModifiers::ALT | KeyModifiers::SUPER)
        {
            return None;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('c') => Some(VimAction::Quit),
                KeyCode::Char('[') => Some(VimAction::Cancel),
                _ => None,
            };
        }
        match key.code {
            KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab => {
                Some(VimAction::Move(Motion::Down, 1))
            }
            KeyCode::Char('k') | KeyCode::Up | KeyCode::BackTab => {
                Some(VimAction::Move(Motion::Up, 1))
            }
            KeyCode::Home => Some(VimAction::FileStart),
            KeyCode::End => Some(VimAction::FileEnd),
            KeyCode::Enter if key.kind == KeyEventKind::Press => Some(VimAction::Submit),
            KeyCode::Esc => Some(VimAction::Cancel),
            KeyCode::Char('q') => Some(VimAction::Quit),
            _ => None,
        }
    }
}
