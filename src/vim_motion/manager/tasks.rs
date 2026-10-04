use super::{KeyCode, KeyEvent, KeyEventKind, Motion, Priority, VimAction, VimManager};

impl VimManager {
    pub(super) fn handle_tasks(&mut self, key: KeyEvent) -> Option<VimAction> {
        if matches!(key.code, KeyCode::Tab | KeyCode::BackTab) {
            self.reset();
            return (key.kind == KeyEventKind::Press).then_some(VimAction::SwitchPane);
        }
        if let KeyCode::Char(c @ '0'..='9') = key.code
            && (c != '0' || self.count > 0)
            && self.task_pending.is_none()
        {
            self.count = (self.count * 10 + usize::from(c as u8 - b'0')).min(9999);
            return None;
        }
        let count = self.count.max(1);
        if let Some(prefix) = self.task_pending.take() {
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
                self.task_pending = Some(c);
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
