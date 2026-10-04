use anyhow::{Context, Result, ensure};
use std::path::PathBuf;

pub(super) fn parse(content: &str) -> Result<PathBuf> {
    let table = content.parse::<toml::Table>()?;
    for key in table.keys() {
        ensure!(key == "database_path", "Unknown setting: {key}");
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
    Ok(path)
}
