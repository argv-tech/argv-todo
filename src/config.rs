use anyhow::{Context, Result, ensure};
use std::{
    fs::{self, OpenOptions},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

const DEFAULT_CONFIG: &str = "# Paths are relative to this config file. --db overrides this setting.\ndatabase_path = \"db.sql\"\n";

pub fn database_path(default_database: &Path, database_override: bool) -> Result<PathBuf> {
    let folder = default_database
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let config_path = folder.join("config.toml");
    ensure!(
        default_database.file_name() != Some(std::ffi::OsStr::new("config.toml")),
        "Database path must differ from config.toml"
    );
    fs::create_dir_all(folder).with_context(|| format!("Could not create {}", folder.display()))?;
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
    if database_override {
        return Ok(default_database.to_path_buf());
    }
    let database = folder.join(configured);
    ensure!(
        database != config_path,
        "Database path must differ from config.toml"
    );
    Ok(database)
}

fn parse(content: &str) -> Result<PathBuf> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_toml_and_rejects_invalid_settings() {
        assert_eq!(parse("").unwrap(), PathBuf::from("db.sql"));
        assert_eq!(
            parse("# local storage\ndatabase_path = 'tasks.sql'").unwrap(),
            PathBuf::from("tasks.sql")
        );
        for content in [
            "database_path = [",
            "database_path = 3",
            "database_path = ''",
            "database_path = '   '",
            "database_path = 'config.toml'",
            "database_path = './config.toml'",
            "databse_path = 'tasks.sql'",
        ] {
            assert!(parse(content).is_err(), "Accepted {content}");
        }
    }

    #[test]
    fn creates_config_preserves_edits_and_resolves_database_paths() {
        let folder = std::env::temp_dir().join(format!(
            "argv-todo-config-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let default_database = folder.join("nested/db.sql");
        let config = folder.join("nested/config.toml");
        assert_eq!(
            database_path(&default_database, false).unwrap(),
            default_database
        );
        assert_eq!(fs::read_to_string(&config).unwrap(), DEFAULT_CONFIG);
        assert!(!default_database.exists());

        let content = "# keep my comment\ndatabase_path = 'storage/tasks.sql'\n";
        fs::write(&config, content).unwrap();
        assert_eq!(
            database_path(&default_database, false).unwrap(),
            folder.join("nested/storage/tasks.sql")
        );
        let override_database = folder.join("nested/custom.sql");
        assert_eq!(
            database_path(&override_database, true).unwrap(),
            override_database
        );
        assert_eq!(fs::read_to_string(&config).unwrap(), content);

        let absolute = folder.join("absolute.sql");
        // Use literal strings so Windows path separators remain unchanged.
        fs::write(&config, format!("database_path = '{}'", absolute.display())).unwrap();
        assert_eq!(database_path(&default_database, false).unwrap(), absolute);

        fs::write(&config, "database_path = [").unwrap();
        let error = database_path(&default_database, false).unwrap_err();
        assert!(error.to_string().contains(&config.display().to_string()));
        assert!(database_path(&override_database, true).is_err());
        assert!(database_path(&config, true).is_err());
        assert_eq!(fs::read_to_string(&config).unwrap(), "database_path = [");
        fs::remove_dir_all(folder).unwrap();
    }
}
