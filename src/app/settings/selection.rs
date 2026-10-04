use super::ConfigSetting;
use crate::app::App;

impl ConfigSetting {
    pub(crate) const ALL: [Self; 6] = [
        Self::DatabasePath,
        Self::TaskView,
        Self::DefaultPriority,
        Self::ShowCompleted,
        Self::SortOrder,
        Self::ShowHints,
    ];

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::DatabasePath => "database_path",
            Self::TaskView => "task_view",
            Self::DefaultPriority => "default_priority",
            Self::ShowCompleted => "show_completed",
            Self::SortOrder => "sort_order",
            Self::ShowHints => "show_hints",
        }
    }

    pub(crate) fn value(self, app: &App) -> &str {
        match self {
            Self::DatabasePath => app
                .config
                .as_ref()
                .map_or("", |config| config.database_path.as_str()),
            Self::TaskView => app.task_view().label(),
            Self::DefaultPriority => app.default_priority().label(),
            Self::ShowCompleted => {
                if app.show_completed() {
                    "true"
                } else {
                    "false"
                }
            }
            Self::SortOrder => app.sort_order().label(),
            Self::ShowHints => {
                if app.show_hints() {
                    "true"
                } else {
                    "false"
                }
            }
        }
    }

    pub(crate) fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|&setting| setting == self)
            .unwrap_or(0)
    }

    pub(super) fn step(self, forward: bool, count: usize, wrap: bool) -> Self {
        let index = self.index();
        let next = if wrap {
            if forward {
                (index + count % Self::ALL.len()) % Self::ALL.len()
            } else {
                (index + Self::ALL.len() - count % Self::ALL.len()) % Self::ALL.len()
            }
        } else if forward {
            index.saturating_add(count).min(Self::ALL.len() - 1)
        } else {
            index.saturating_sub(count)
        };
        Self::ALL[next]
    }
}
