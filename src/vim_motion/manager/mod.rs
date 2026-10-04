mod help;
mod input;
mod pending;
mod tasks;

use crate::db::Priority;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::{
    VimMode,
    vim_action::{
        EditAction, EditTarget, InputTarget, InsertPosition, Motion, Operator, TextObject,
        VimAction,
    },
};
use pending::Pending;

/// Input focus stays active when a field switches to Normal or Visual mode.
#[derive(Default)]
pub(crate) struct VimManager {
    mode: VimMode,
    input: Option<InputTarget>,
    count: usize,
    task_pending: Option<char>,
    pending: Option<Pending>,
    sequence: String,
}

impl VimManager {
    pub(crate) fn mode(&self) -> VimMode {
        self.mode
    }

    pub(crate) fn input(&self) -> Option<InputTarget> {
        self.input
    }

    pub(crate) fn begin_input(&mut self, target: InputTarget) {
        self.input = Some(target);
        self.set_mode(VimMode::Insert);
    }

    pub(crate) fn end_input(&mut self) {
        self.input = None;
        self.set_mode(VimMode::Normal);
    }

    pub(crate) fn set_mode(&mut self, mode: VimMode) {
        self.mode = mode;
        self.reset();
    }

    fn reset(&mut self) {
        self.count = 0;
        self.task_pending = None;
        self.pending = None;
        self.sequence.clear();
    }

    pub(crate) fn pending_label(&self) -> String {
        if self.input.is_some() {
            return self.sequence.clone();
        }
        let count = if self.count == 0 {
            String::new()
        } else {
            self.count.to_string()
        };
        format!(
            "{count}{}",
            self.task_pending.map_or(String::new(), |c| c.to_string())
        )
    }

    fn escape(&mut self) -> VimAction {
        let editing =
            self.input.is_some() && (self.mode != VimMode::Normal || !self.sequence.is_empty());
        self.reset();
        if editing {
            VimAction::Input(EditAction::Normal)
        } else {
            VimAction::Cancel
        }
    }

    pub(crate) fn handle(&mut self, key: KeyEvent) -> Option<VimAction> {
        if key.kind == KeyEventKind::Release {
            return None;
        }
        if key.code == KeyCode::Esc
            || (key.code == KeyCode::Char('[') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            return Some(self.escape());
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            let action = match key.code {
                KeyCode::Char('c') => Some(VimAction::Quit),
                KeyCode::Char('w') if self.input.is_some() && self.mode.typing() => {
                    Some(VimAction::DeleteWord)
                }
                KeyCode::Char('u') if self.input.is_some() && self.mode.typing() => {
                    Some(VimAction::Clear)
                }
                KeyCode::Char('h') if self.input.is_some() && self.mode.typing() => {
                    Some(VimAction::Backspace)
                }
                KeyCode::Right if self.input.is_some() => {
                    Some(VimAction::Move(Motion::WordForward, 1))
                }
                KeyCode::Left if self.input.is_some() => {
                    Some(VimAction::Move(Motion::WordBackward, 1))
                }
                KeyCode::Char('r') if self.input.is_some() && !self.mode.typing() => {
                    Some(VimAction::Input(EditAction::Redo(self.count.max(1))))
                }
                KeyCode::Char('r') if self.input.is_none() => Some(VimAction::Refresh),
                KeyCode::Char('p')
                    if self.input != Some(InputTarget::Search)
                        && self.input != Some(InputTarget::DatabasePath) =>
                {
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
            self.reset();
            return None;
        }
        if self.input.is_some() {
            self.handle_input(key)
        } else {
            self.handle_tasks(key)
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/vim_motion/manager.rs"]
mod tests;
