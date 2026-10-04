use super::pending::Pending;
use super::{
    EditAction, EditTarget, InsertPosition, KeyCode, KeyEvent, Motion, Operator, VimAction,
    VimManager,
};

impl VimManager {
    pub(super) fn handle_input(&mut self, key: KeyEvent) -> Option<VimAction> {
        if key.code == KeyCode::Enter {
            self.reset();
            return Some(VimAction::Submit);
        }
        if self.mode.typing() {
            return match key.code {
                KeyCode::Char(c) => Some(VimAction::Insert(c)),
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
            && !matches!(self.pending, Some(Pending::Find(..) | Pending::Replace(..)))
        {
            self.count = (self.count * 10 + usize::from(c as u8 - b'0')).min(9999);
            self.sequence.push(c);
            return None;
        }
        let count = self.count.max(1);
        if let Some(pending) = self.pending.take() {
            let action = self.complete(pending, key.code, count);
            if self.pending.is_none() {
                self.reset();
            }
            return action.map(VimAction::Input);
        }
        let visual = self.mode.visual();
        let selection = EditTarget::Selection;
        let action = match key.code {
            KeyCode::Char('d' | 'x') | KeyCode::Delete if visual => {
                EditAction::Operate(Operator::Delete, selection, 1)
            }
            KeyCode::Char('c' | 's') if visual => {
                EditAction::Operate(Operator::Change, selection, 1)
            }
            KeyCode::Char('y') if visual => EditAction::Operate(Operator::Yank, selection, 1),
            KeyCode::Char('u') if visual => EditAction::Operate(Operator::Lowercase, selection, 1),
            KeyCode::Char('U') if visual => EditAction::Operate(Operator::Uppercase, selection, 1),
            KeyCode::Char('~') if visual => EditAction::Operate(Operator::ToggleCase, selection, 1),
            KeyCode::Char('i' | 'a') if visual => {
                self.queue(
                    Pending::Object(None, key.code == KeyCode::Char('a'), count),
                    key.code,
                );
                return None;
            }
            KeyCode::Char(c @ ('d' | 'c' | 'y')) => {
                let operator = match c {
                    'd' => Operator::Delete,
                    'c' => Operator::Change,
                    _ => Operator::Yank,
                };
                self.queue(Pending::Operator(operator, count), key.code);
                return None;
            }
            KeyCode::Char('g') => {
                self.queue(Pending::G(None, count), key.code);
                return None;
            }
            KeyCode::Char(c @ ('f' | 'F' | 't' | 'T')) => {
                self.queue(
                    Pending::Find(None, c.is_uppercase(), matches!(c, 't' | 'T'), count),
                    key.code,
                );
                return None;
            }
            KeyCode::Char('r') => {
                self.queue(Pending::Replace(count), key.code);
                return None;
            }
            KeyCode::Char('R') => EditAction::ReplaceMode,
            KeyCode::Char('i') => EditAction::Insert(InsertPosition::Cursor, count),
            KeyCode::Char('a') => EditAction::Insert(InsertPosition::After, count),
            KeyCode::Char('I') => EditAction::Insert(InsertPosition::FirstNonBlank, count),
            KeyCode::Char('A') => EditAction::Insert(InsertPosition::End, count),
            KeyCode::Char('v') => EditAction::Visual(false),
            KeyCode::Char('V') => EditAction::Visual(true),
            KeyCode::Char('o' | 'O') if visual => EditAction::SwapAnchor,
            KeyCode::Char('x') | KeyCode::Delete => {
                EditAction::Operate(Operator::Delete, EditTarget::Motion(Motion::Right), count)
            }
            KeyCode::Char('X') => {
                EditAction::Operate(Operator::Delete, EditTarget::Motion(Motion::Left), count)
            }
            KeyCode::Char('D') => EditAction::Operate(
                Operator::Delete,
                if visual {
                    EditTarget::Line
                } else {
                    EditTarget::Motion(Motion::LineEnd)
                },
                count,
            ),
            KeyCode::Char('C') => EditAction::Operate(
                Operator::Change,
                if visual {
                    EditTarget::Line
                } else {
                    EditTarget::Motion(Motion::LineEnd)
                },
                count,
            ),
            KeyCode::Char('Y') => EditAction::Operate(Operator::Yank, EditTarget::Line, count),
            KeyCode::Char('s') => {
                EditAction::Operate(Operator::Change, EditTarget::Motion(Motion::Right), count)
            }
            KeyCode::Char('S') => EditAction::Operate(Operator::Change, EditTarget::Line, count),
            KeyCode::Char('~') => EditAction::Operate(
                Operator::ToggleCase,
                EditTarget::Motion(Motion::Right),
                count,
            ),
            KeyCode::Char('p' | 'P') => EditAction::Put(key.code == KeyCode::Char('P'), count),
            KeyCode::Char('u') => EditAction::Undo(count),
            KeyCode::Char('.') => EditAction::Repeat(count),
            code => {
                let motion = motion(code);
                self.reset();
                return motion.map(|motion| VimAction::Move(motion, count));
            }
        };
        self.reset();
        Some(VimAction::Input(action))
    }

    pub(super) fn queue(&mut self, pending: Pending, key: KeyCode) {
        self.pending = Some(pending);
        self.count = 0;
        if let KeyCode::Char(c) = key {
            self.sequence.push(c);
        }
    }
}

pub(super) fn motion(code: KeyCode) -> Option<Motion> {
    Some(match code {
        KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => Motion::Left,
        KeyCode::Char('l' | ' ') | KeyCode::Right => Motion::Right,
        KeyCode::Char('j') | KeyCode::Down => Motion::Down,
        KeyCode::Char('k') | KeyCode::Up => Motion::Up,
        KeyCode::Char('w') => Motion::WordForward,
        KeyCode::Char('b') => Motion::WordBackward,
        KeyCode::Char('e') => Motion::WordEnd,
        KeyCode::Char('W') => Motion::BigWordForward,
        KeyCode::Char('B') => Motion::BigWordBackward,
        KeyCode::Char('E') => Motion::BigWordEnd,
        KeyCode::Char('0') | KeyCode::Home => Motion::LineStart,
        KeyCode::Char('^' | '_') => Motion::FirstNonBlank,
        KeyCode::Char('$' | 'G') | KeyCode::End => Motion::LineEnd,
        KeyCode::Char('|') => Motion::Column,
        KeyCode::Char('%') => Motion::MatchingBracket,
        KeyCode::Char(';') => Motion::RepeatFind(false),
        KeyCode::Char(',') => Motion::RepeatFind(true),
        _ => return None,
    })
}
