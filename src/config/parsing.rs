use super::TaskView;
use anyhow::{Context, Result, ensure};
use std::path::PathBuf;

pub(super) struct Settings {
    pub(super) database_path: PathBuf,
    pub(super) task_view: TaskView,
}

pub(super) fn parse(content: &str) -> Result<Settings> {
    let table = content.parse::<toml::Table>()?;
    for key in table.keys() {
        ensure!(
            matches!(key.as_str(), "database_path" | "task_view"),
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
    })
}
