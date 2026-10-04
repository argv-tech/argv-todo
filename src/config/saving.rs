use anyhow::{Context, Result};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use super::{Config, TaskView, parsing::parse};

impl Config {
    pub(crate) fn save_database_path(&mut self, path: &str) -> Result<()> {
        self.save_setting("database_path", path)?;
        self.database_path = path.into();
        Ok(())
    }

    pub(crate) fn save_task_view(&mut self, view: TaskView) -> Result<()> {
        self.save_setting("task_view", view.label())?;
        self.task_view = view;
        Ok(())
    }

    fn save_setting(&self, setting: &str, value: &str) -> Result<()> {
        let mut content = fs::read_to_string(&self.path)
            .with_context(|| format!("Could not read config {}", self.path.display()))?;
        parse(&content).with_context(|| format!("Invalid config {}", self.path.display()))?;
        let table = toml::de::DeTable::parse(&content)?;
        let span = table
            .get_ref()
            .iter()
            .find_map(|(key, value)| (key.get_ref().as_ref() == setting).then(|| value.span()));
        let value = toml::Value::String(value.into()).to_string();
        if let Some(span) = span {
            content.replace_range(span, &value);
        } else {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(&format!("{setting} = {value}\n"));
        }
        parse(&content)?;
        write_config(&self.path, &content)?;
        Ok(())
    }
}

fn write_config(path: &Path, content: &str) -> Result<()> {
    let temporary = path.with_file_name(format!(".config.toml.{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .with_context(|| format!("Could not save config {}", path.display()))?;
    let result = (|| -> std::io::Result<()> {
        file.set_permissions(fs::metadata(path)?.permissions())?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.with_context(|| format!("Could not save config {}", path.display()))
}
