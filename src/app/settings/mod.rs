mod selection;
mod updates;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfigSetting {
    #[default]
    DatabasePath,
    TaskView,
    DefaultPriority,
    ShowCompleted,
    SortOrder,
    ShowHints,
}
