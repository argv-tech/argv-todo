mod selection;
mod updates;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfigCategory {
    Storage,
    Display,
    TaskBehavior,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfigSetting {
    #[default]
    DatabasePath,
    TaskView,
    DefaultPriority,
    ShowCompleted,
    SortOrder,
    CompletedDelete,
    ShowHints,
}
