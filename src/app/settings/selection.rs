use super::{ConfigCategory, ConfigSetting};
use crate::app::App;

impl ConfigSetting {
    pub(crate) const ALL: [Self; 10] = [
        Self::DatabasePath,
        Self::TaskView,
        Self::ShowCompleted,
        Self::ShowHints,
        Self::DefaultPriority,
        Self::SortOrder,
        Self::CompletedDelete,
        Self::Notifications,
        Self::NotifyReminder1Minutes,
        Self::NotifyReminder2Minutes,
    ];

    pub(crate) fn category(self) -> ConfigCategory {
        match self {
            Self::DatabasePath => ConfigCategory::Storage,
            Self::TaskView | Self::ShowCompleted | Self::ShowHints => ConfigCategory::Display,
            Self::DefaultPriority | Self::SortOrder | Self::CompletedDelete => {
                ConfigCategory::TaskBehavior
            }
            Self::Notifications | Self::NotifyReminder1Minutes | Self::NotifyReminder2Minutes => {
                ConfigCategory::Notifications
            }
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::DatabasePath => "database_path",
            Self::TaskView => "task_view",
            Self::DefaultPriority => "default_priority",
            Self::ShowCompleted => "show_completed",
            Self::SortOrder => "sort_order",
            Self::ShowHints => "show_hints",
            Self::CompletedDelete => "completed_delete",
            Self::Notifications => "notifications",
            Self::NotifyReminder1Minutes => "notify_reminder_1_minutes",
            Self::NotifyReminder2Minutes => "notify_reminder_2_minutes",
        }
    }

    pub(crate) fn value(self, app: &App) -> String {
        match self {
            Self::DatabasePath => app
                .config
                .as_ref()
                .map_or(String::new(), |config| config.database_path.clone()),
            Self::TaskView => app.task_view().label().to_owned(),
            Self::DefaultPriority => app.default_priority().label().to_owned(),
            Self::ShowCompleted => {
                if app.show_completed() {
                    "true".to_owned()
                } else {
                    "false".to_owned()
                }
            }
            Self::SortOrder => app.sort_order().label().to_owned(),
            Self::CompletedDelete => app.completed_delete().label().to_owned(),
            Self::ShowHints => {
                if app.show_hints() {
                    "true".to_owned()
                } else {
                    "false".to_owned()
                }
            }
            Self::Notifications => {
                if app.notifications() {
                    "true".to_owned()
                } else {
                    "false".to_owned()
                }
            }
            Self::NotifyReminder1Minutes => app.notify_reminder_1_minutes().to_string(),
            Self::NotifyReminder2Minutes => app.notify_reminder_2_minutes().to_string(),
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
