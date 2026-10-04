use crate::db::Priority;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Motion {
    Left,
    Down,
    Up,
    Right,
    WordForward,
    WordBackward,
    WordEnd,
    BigWordForward,
    BigWordBackward,
    BigWordEnd,
    WordEndBackward,
    BigWordEndBackward,
    LineStart,
    FirstNonBlank,
    LineEnd,
    Column,
    MatchingBracket,
    Find {
        character: char,
        backward: bool,
        till: bool,
    },
    RepeatFind(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputTarget {
    Task,
    Search,
    DatabasePath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Operator {
    Delete,
    Change,
    Yank,
    Lowercase,
    Uppercase,
    ToggleCase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextObject {
    Word(bool),
    Delimited(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditTarget {
    Motion(Motion),
    Object(TextObject, bool),
    Line,
    Selection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InsertPosition {
    Cursor,
    After,
    FirstNonBlank,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditAction {
    Move(Motion, usize),
    Normal,
    Insert(InsertPosition, usize),
    Visual(bool),
    SwapAnchor,
    Select(TextObject, bool, usize),
    Operate(Operator, EditTarget, usize),
    Replace(char, usize),
    ReplaceMode,
    Put(bool, usize),
    Undo(usize),
    Redo(usize),
    Repeat(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VimAction {
    Move(Motion, usize),
    FileStart,
    FileEnd,
    PageUp,
    PageDown,
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
    Input(EditAction),
    Quit,
}
