use super::{SortOrder, TaskView};
use crate::db::Priority;
use anyhow::{Context, Result, ensure};
use std::path::PathBuf;

pub(super) struct Settings {
    pub(super) database_path: PathBuf,
    pub(super) task_view: TaskView,
    pub(super) default_priority: Priority,
    pub(super) show_completed: bool,
    pub(super) sort_order: SortOrder,
    pub(super) show_hints: bool,
}

pub(super) fn parse(content: &str) -> Result<Settings> {
    let table = content.parse::<toml::Table>()?;
    for key in table.keys() {
        ensure!(
            matches!(
                key.as_str(),
                "database_path"
                    | "task_view"
                    | "default_priority"
                    | "show_completed"
                    | "sort_order"
                    | "show_hints"
            ),
            "Unknown setting: {key}"
        );
    }
    let path = match table.get("database_path") {
        Some(value) => value.as_str().context("database_path must be a string")?,
        None => "db.sql",
    };
    ensure!(!path.trim().is_empty(), "database_path cannot be empty");
    let path = PathBuf::from(path);
    ensure!(
        path.file_name() != Some(std::ffi::OsStr::new("config.toml")),
        "Database path must differ from config.toml"
    );
    let task_view = match table.get("task_view") {
        Some(value) => TaskView::parse(value.as_str().context("task_view must be a string")?)?,
        None => TaskView::Normal,
    };
    Ok(Settings {
        database_path: path,
        task_view,
        default_priority: match table.get("default_priority") {
            None => Priority::Mid,
            Some(value) => match value
                .as_str()
                .context("default_priority must be a string")?
            {
                "high" => Priority::High,
                "mid" => Priority::Mid,
                "low" => Priority::Low,
                _ => anyhow::bail!("default_priority must be high, mid, or low"),
            },
        },
        show_completed: boolean(&table, "show_completed", true)?,
        sort_order: match table.get("sort_order") {
            None => SortOrder::Priority,
            Some(value) => {
                SortOrder::parse(value.as_str().context("sort_order must be a string")?)?
            }
        },
        show_hints: boolean(&table, "show_hints", true)?,
    })
}

fn boolean(table: &toml::Table, key: &str, default: bool) -> Result<bool> {
    match table.get(key) {
        None => Ok(default),
        Some(value) => value
            .as_bool()
            .with_context(|| format!("{key} must be a boolean")),
    }
}
