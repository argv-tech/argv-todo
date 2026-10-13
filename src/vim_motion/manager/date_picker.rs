use super::{DateUnit, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, VimAction, VimManager};

impl VimManager {
    pub(crate) fn handle_date_picker(&mut self, key: KeyEvent) -> Option<VimAction> {
        if key.kind == KeyEventKind::Release {
            return None;
        }
        if key.code == KeyCode::Esc {
            self.reset();
            return Some(VimAction::Cancel);
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            self.reset();
            return match key.code {
                KeyCode::Char('c') => Some(VimAction::Quit),
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
        if let KeyCode::Char(c @ '0'..='9') = key.code
            && (c != '0' || self.count > 0)
            && key.modifiers.is_empty()
        {
            self.count = (self.count * 10 + usize::from(c as u8 - b'0')).min(9999);
            return None;
        }
        // Held keys repeat day and week steps; month jumps and Enter stay single-shot.
        let press = key.kind == KeyEventKind::Press;
        let steps = self.count.clamp(1, 9999) as i32;
        let action = match (key.code, key.modifiers) {
            (KeyCode::Char('h'), KeyModifiers::NONE) | (KeyCode::Left, _) => {
                Some(VimAction::ShiftDate(DateUnit::Day, -steps))
            }
            (KeyCode::Char('l'), KeyModifiers::NONE) | (KeyCode::Right, _) => {
                Some(VimAction::ShiftDate(DateUnit::Day, steps))
            }
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                Some(VimAction::ShiftDate(DateUnit::Week, steps))
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                Some(VimAction::ShiftDate(DateUnit::Week, -steps))
            }
            (KeyCode::Char('H'), _) | (KeyCode::Char('h'), KeyModifiers::SHIFT) => {
                press.then_some(VimAction::ShiftDate(DateUnit::Month, -steps))
            }
            (KeyCode::Char('L'), _) | (KeyCode::Char('l'), KeyModifiers::SHIFT) => {
                press.then_some(VimAction::ShiftDate(DateUnit::Month, steps))
            }
            (KeyCode::Char('t'), KeyModifiers::NONE) => press.then_some(VimAction::PickToday),
            (KeyCode::Char('x'), KeyModifiers::NONE) => press.then_some(VimAction::ClearDeadline),
            (KeyCode::Tab, _) => Some(VimAction::FocusNext),
            (KeyCode::BackTab, _) => Some(VimAction::FocusPrev),
            (KeyCode::Enter, KeyModifiers::NONE) => press.then_some(VimAction::Submit),
            (KeyCode::Char('q'), KeyModifiers::NONE) => Some(VimAction::Quit),
            _ => None,
        };
        self.reset();
        action
    }
}
