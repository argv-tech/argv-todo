use crate::db::{Priority, TaskMove};

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
    DocumentStart,
    DocumentEnd,
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
    Note,
    Search,
    DatabasePath,
    ProjectName,
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
    MoveTask(TaskMove, usize),
    FileStart,
    FileEnd,
    PageUp,
    PageDown,
    SwitchPane,
    SwitchPaneBackward,
    ProjectRight(usize),
    ProjectLeft(usize),
    ConvertProject,
    ClearCompleted,
    ShowDeleted,
    RestoreArchived,
    ClearArchived,
    AddProject,
    RenameProject,
    DeleteProject,
    AssignProject,
    AddChild,
    AddBelow,
    AddAbove,
    Edit,
    EditNote,
    Priority(Priority),
    CyclePriority,
    Toggle,
    ToggleCollapse,
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
