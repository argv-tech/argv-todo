use crate::db::Priority;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Left,
    Down,
    Up,
    Right,
    WordForward,
    WordBackward,
    LineStart,
    LineEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimAction {
    Move(Motion, usize),
    FileStart,
    FileEnd,
    AddChild,
    AddBelow,
    AddAbove,
    Edit,
    Priority(Priority),
    CyclePriority,
    Toggle,
    Delete(usize),
    Undo,
    Search,
    Help,
    Refresh,
    Submit,
    Cancel,
    Insert(char),
    Backspace,
    DeleteChar,
    DeleteWord,
    Clear,
    Quit,
}
