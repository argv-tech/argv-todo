use super::{EditAction, EditTarget, Editor, Motion, VimAction, VimMode};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone)]
pub(super) struct Snapshot {
    pub(super) text: String,
    pub(super) cursor: usize,
}

#[derive(Clone)]
pub(super) enum RecordedAction {
    Key(VimAction),
    Paste(String),
}

pub(super) struct Change {
    pub(super) before: Snapshot,
    pub(super) actions: Vec<RecordedAction>,
}

impl Editor {
    pub(crate) fn prepare_submit(&mut self) {
        self.finish_change();
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            text: self.text.clone(),
            cursor: self.cursor,
        }
    }

    pub(crate) fn apply(&mut self, action: VimAction) {
        match action {
            VimAction::Input(EditAction::Normal) => {
                self.finish_change();
                if self.mode.typing() {
                    self.cursor = self.previous_at(self.cursor);
                }
                self.mode = VimMode::Normal;
                self.anchor = None;
                self.replace_history.clear();
                self.normalize_cursor();
            }
            VimAction::Input(EditAction::Undo(count)) => self.restore_history(false, count),
            VimAction::Input(EditAction::Redo(count)) => self.restore_history(true, count),
            VimAction::Input(EditAction::Repeat(count)) => self.repeat_change(count),
            _ => self.record(RecordedAction::Key(action)),
        }
    }

    pub(crate) fn insert(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.record(RecordedAction::Paste(text.to_owned()));
    }

    fn record(&mut self, action: RecordedAction) {
        let before = self.snapshot();
        let mode = self.mode;
        let repeat_action = self.repeatable_action(&action);
        self.replay_action(&action);
        let changed = before.text != self.text;
        if let Some(change) = &mut self.change {
            change.actions.push(action);
        } else if self.mode.typing() && (changed || self.mode != mode) {
            self.change = Some(Change {
                before,
                actions: vec![repeat_action],
            });
        } else if changed {
            self.undo.push(before);
            self.redo.clear();
            self.last_change = vec![repeat_action];
        }
    }

    fn repeatable_action(&self, action: &RecordedAction) -> RecordedAction {
        let Some(range) = self.selection() else {
            return action.clone();
        };
        let count = self.text[range].graphemes(true).count();
        match action {
            RecordedAction::Key(VimAction::Input(EditAction::Operate(
                operator,
                EditTarget::Selection,
                _,
            ))) => {
                let target = if self.mode == VimMode::VisualLine {
                    EditTarget::Line
                } else {
                    EditTarget::Motion(Motion::Right)
                };
                RecordedAction::Key(VimAction::Input(EditAction::Operate(
                    *operator, target, count,
                )))
            }
            RecordedAction::Key(VimAction::Input(EditAction::Replace(character, _))) => {
                RecordedAction::Key(VimAction::Input(EditAction::Replace(*character, count)))
            }
            _ => action.clone(),
        }
    }

    fn replay_action(&mut self, action: &RecordedAction) {
        match action {
            RecordedAction::Key(action) => self.apply_inner(*action),
            RecordedAction::Paste(text) => self.paste_inner(text),
        }
        self.normalize_cursor();
    }

    fn finish_change(&mut self) {
        let Some(mut change) = self.change.take() else {
            return;
        };
        if self.insert_count > 1 {
            let typing = if matches!(
                change.actions.first(),
                Some(RecordedAction::Key(VimAction::Input(EditAction::Insert(
                    ..
                ))))
            ) {
                change.actions[1..].to_vec()
            } else {
                change.actions.clone()
            };
            for _ in 1..self.insert_count {
                for action in &typing {
                    self.replay_action(action);
                }
                change.actions.extend_from_slice(&typing);
            }
        }
        self.insert_count = 1;
        if self.text != change.before.text {
            self.undo.push(change.before);
            self.redo.clear();
            change
                .actions
                .push(RecordedAction::Key(VimAction::Input(EditAction::Normal)));
            self.last_change = change.actions;
        }
    }

    fn restore_history(&mut self, redo: bool, count: usize) {
        self.finish_change();
        self.mode = VimMode::Normal;
        self.anchor = None;
        for _ in 0..count {
            let snapshot = if redo {
                self.redo.pop()
            } else {
                self.undo.pop()
            };
            let Some(snapshot) = snapshot else {
                break;
            };
            let current = self.snapshot();
            if redo {
                self.undo.push(current);
            } else {
                self.redo.push(current);
            }
            self.text = snapshot.text;
            self.cursor = snapshot.cursor;
            self.normalize_cursor();
        }
    }

    fn repeat_change(&mut self, count: usize) {
        let actions = self.last_change.clone();
        if actions.is_empty() {
            return;
        }
        let before = self.snapshot();
        for _ in 0..count {
            for action in &actions {
                self.replay_action(action);
            }
        }
        self.insert_count = 1;
        if self.text != before.text {
            self.undo.push(before);
            self.redo.clear();
        }
    }
}
