use super::input::motion;
use super::{EditAction, EditTarget, KeyCode, Motion, Operator, TextObject, VimManager};

#[derive(Clone, Copy)]
pub(super) enum Pending {
    Operator(Operator, usize),
    Object(Option<Operator>, bool, usize),
    Find(Option<Operator>, bool, bool, usize),
    Replace(usize),
    G(Option<Operator>, usize),
}

impl VimManager {
    pub(super) fn complete(
        &mut self,
        pending: Pending,
        key: KeyCode,
        count: usize,
    ) -> Option<EditAction> {
        match pending {
            Pending::Replace(count) => match key {
                KeyCode::Char(c) => Some(EditAction::Replace(c, count)),
                _ => None,
            },
            Pending::Find(operator, backward, till, prefix_count) => {
                let KeyCode::Char(character) = key else {
                    return None;
                };
                let motion = Motion::Find {
                    character,
                    backward,
                    till,
                };
                Some(command_motion(operator, motion, prefix_count))
            }
            Pending::Object(operator, around, prefix_count) => {
                let object = match key {
                    KeyCode::Char('w') => TextObject::Word(false),
                    KeyCode::Char('W') => TextObject::Word(true),
                    KeyCode::Char(c @ ('(' | ')' | 'b')) => {
                        TextObject::Delimited(if c == 'b' { '(' } else { c })
                    }
                    KeyCode::Char(c @ ('{' | '}' | 'B')) => {
                        TextObject::Delimited(if c == 'B' { '{' } else { c })
                    }
                    KeyCode::Char(c @ ('[' | ']' | '<' | '>' | '\'' | '"' | '`')) => {
                        TextObject::Delimited(c)
                    }
                    _ => return None,
                };
                let count = prefix_count.saturating_mul(count).min(9999);
                Some(match operator {
                    Some(operator) => {
                        EditAction::Operate(operator, EditTarget::Object(object, around), count)
                    }
                    None => EditAction::Select(object, around, count),
                })
            }
            Pending::Operator(operator, prefix_count) => {
                let count = prefix_count.saturating_mul(count).min(9999);
                let doubled = match operator {
                    Operator::Delete => 'd',
                    Operator::Change => 'c',
                    Operator::Yank => 'y',
                    Operator::Lowercase => 'u',
                    Operator::Uppercase => 'U',
                    Operator::ToggleCase => '~',
                };
                if key == KeyCode::Char(doubled) {
                    return Some(EditAction::Operate(operator, EditTarget::Line, count));
                }
                match key {
                    KeyCode::Char('i' | 'a') => self.queue(
                        Pending::Object(Some(operator), key == KeyCode::Char('a'), count),
                        key,
                    ),
                    KeyCode::Char('g') => self.queue(Pending::G(Some(operator), count), key),
                    KeyCode::Char(c @ ('f' | 'F' | 't' | 'T')) => self.queue(
                        Pending::Find(
                            Some(operator),
                            c.is_uppercase(),
                            matches!(c, 't' | 'T'),
                            count,
                        ),
                        key,
                    ),
                    code => {
                        return motion(code)
                            .map(|motion| command_motion(Some(operator), motion, count));
                    }
                }
                None
            }
            Pending::G(operator, prefix_count) => {
                let count = prefix_count.saturating_mul(count).min(9999);
                match key {
                    KeyCode::Char('g' | '0') => {
                        Some(command_motion(operator, Motion::LineStart, count))
                    }
                    KeyCode::Char('e') => {
                        Some(command_motion(operator, Motion::WordEndBackward, count))
                    }
                    KeyCode::Char('E') => {
                        Some(command_motion(operator, Motion::BigWordEndBackward, count))
                    }
                    KeyCode::Char('$') => Some(command_motion(operator, Motion::LineEnd, count)),
                    KeyCode::Char(c @ ('u' | 'U' | '~')) if operator.is_none() => {
                        let operator = match c {
                            'u' => Operator::Lowercase,
                            'U' => Operator::Uppercase,
                            _ => Operator::ToggleCase,
                        };
                        if self.mode.visual() {
                            return Some(EditAction::Operate(operator, EditTarget::Selection, 1));
                        }
                        self.queue(Pending::Operator(operator, count), key);
                        None
                    }
                    _ => None,
                }
            }
        }
    }
}

fn command_motion(operator: Option<Operator>, motion: Motion, count: usize) -> EditAction {
    match operator {
        Some(operator) => EditAction::Operate(operator, EditTarget::Motion(motion), count),
        None => EditAction::Move(motion, count),
    }
}
