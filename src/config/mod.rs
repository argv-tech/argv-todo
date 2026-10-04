mod parsing;
mod saving;

#[cfg(test)]
#[path = "../../tests/unit/config.rs"]
mod tests;

use anyhow::{Context, Result, ensure};
use std::{
    fs::{self, OpenOptions},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

use self::parsing::parse;

const DEFAULT_CONFIG: &str = "# Paths are relative to this config file. --db overrides this setting.\ndatabase_path = \"db.sql\"\n";

pub(crate) struct Config {
    pub(crate) path: PathBuf,
    pub(crate) database_path: String,
    pub(crate) active_database: PathBuf,
    pub(crate) database_override: bool,
}

impl Config {
    pub(crate) fn load(default_database: &Path, database_override: bool) -> Result<Self> {
        let folder = default_database
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let config_path = folder.join("config.toml");
        ensure!(
            default_database.file_name() != Some(std::ffi::OsStr::new("config.toml")),
            "Database path must differ from config.toml"
        );
        fs::create_dir_all(folder)
            .with_context(|| format!("Could not create {}", folder.display()))?;
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&config_path)
        {
            Ok(mut file) => file
                .write_all(DEFAULT_CONFIG.as_bytes())
                .with_context(|| format!("Could not write config {}", config_path.display()))?,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("Could not create config {}", config_path.display()));
            }
        }
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Could not read config {}", config_path.display()))?;
        let configured =
            parse(&content).with_context(|| format!("Invalid config {}", config_path.display()))?;
        let database = if database_override {
            default_database.to_path_buf()
        } else {
            folder.join(&configured)
        };
        ensure!(
            database != config_path,
            "Database path must differ from config.toml"
        );
        Ok(Self {
            path: config_path,
            database_path: configured.to_string_lossy().into_owned(),
            active_database: database,
            database_override,
        })
    }

    pub(crate) fn reload(&mut self) -> Result<()> {
        let content = fs::read_to_string(&self.path)
            .with_context(|| format!("Could not read config {}", self.path.display()))?;
        self.database_path = parse(&content)
            .with_context(|| format!("Invalid config {}", self.path.display()))?
            .to_string_lossy()
            .into_owned();
        Ok(())
    }
}
