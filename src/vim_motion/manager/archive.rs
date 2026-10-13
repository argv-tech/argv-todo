use super::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, VimAction, VimManager};

impl VimManager {
    pub(crate) fn handle_archive(&mut self, key: KeyEvent) -> Option<VimAction> {
        if self.input.is_some() {
            return self.handle(key);
        }
        let action = match (key.code, key.modifiers) {
            (KeyCode::Char('r'), KeyModifiers::NONE) => Some(VimAction::RestoreArchived),
            (KeyCode::Char('D'), KeyModifiers::NONE | KeyModifiers::SHIFT)
            | (KeyCode::Char('d'), KeyModifiers::SHIFT) => Some(VimAction::ClearArchived),
            (KeyCode::Enter, KeyModifiers::NONE) => Some(VimAction::Submit),
            _ => None,
        };
        if let Some(action) = action {
            self.reset();
            return (key.kind == KeyEventKind::Press).then_some(action);
        }
        self.handle(key)
    }
}
