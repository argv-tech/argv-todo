use anyhow::{Context, Result, ensure};
use std::{
    fs::{self, OpenOptions},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

const DEFAULT_CONFIG: &str = "# Paths are relative to this config file. --db overrides this setting.\ndatabase_path = \"db.sql\"\n";

pub struct Config {
    pub path: PathBuf,
    pub database_path: String,
    pub active_database: PathBuf,
    pub database_override: bool,
}

impl Config {
    pub fn load(default_database: &Path, database_override: bool) -> Result<Self> {
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

    pub fn reload(&mut self) -> Result<()> {
        let content = fs::read_to_string(&self.path)
            .with_context(|| format!("Could not read config {}", self.path.display()))?;
        self.database_path = parse(&content)
            .with_context(|| format!("Invalid config {}", self.path.display()))?
            .to_string_lossy()
            .into_owned();
        Ok(())
    }

    pub fn save_database_path(&mut self, path: &str) -> Result<()> {
        let mut content = fs::read_to_string(&self.path)
            .with_context(|| format!("Could not read config {}", self.path.display()))?;
        parse(&content).with_context(|| format!("Invalid config {}", self.path.display()))?;
        let table = toml::de::DeTable::parse(&content)?;
        let span = table.get_ref().iter().find_map(|(key, value)| {
            (key.get_ref().as_ref() == "database_path").then(|| value.span())
        });
        let value = toml::Value::String(path.into()).to_string();
        if let Some(span) = span {
            content.replace_range(span, &value);
        } else {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(&format!("database_path = {value}\n"));
        }
        parse(&content)?;
        write_config(&self.path, &content)?;
        self.database_path = path.into();
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

    fn database_path(path: &Path, overridden: bool) -> Result<PathBuf> {
        Ok(Config::load(path, overridden)?.active_database)
    }

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

    #[test]
    fn saves_paths_preserving_comments_and_keeps_current_database_on_failure() {
        let folder = std::env::temp_dir().join(format!(
            "argv-todo-config-save-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let database = folder.join("db.sql");
        let mut config = Config::load(&database, false).unwrap();
        for value in [
            "'db.sql'",
            "\"db.sql\"",
            "'''\ndb.sql'''",
            "\"\"\"\ndb.sql\"\"\"",
        ] {
            fs::write(
                &config.path,
                format!("# header\n\"database_path\" = {value} # inline\n# footer\n"),
            )
            .unwrap();
            let new_path = "storage/界 'quoted' \\tasks.sql";
            config.save_database_path(new_path).unwrap();
            let saved = fs::read_to_string(&config.path).unwrap();
            assert_eq!(parse(&saved).unwrap(), PathBuf::from(new_path));
            assert!(saved.starts_with("# header\n\"database_path\" = "));
            assert!(saved.ends_with(" # inline\n# footer\n"));
            assert_eq!(config.database_path, new_path);
            assert_eq!(config.active_database, database);
        }
        fs::write(&config.path, "# no settings yet").unwrap();
        config.save_database_path("tasks.sql").unwrap();
        let saved = fs::read_to_string(&config.path).unwrap();
        assert!(saved.starts_with("# no settings yet\n"));
        assert_eq!(parse(&saved).unwrap(), PathBuf::from("tasks.sql"));
        for invalid in ["", " ", "config.toml"] {
            assert!(config.save_database_path(invalid).is_err());
            assert_eq!(fs::read_to_string(&config.path).unwrap(), saved);
            assert_eq!(config.database_path, "tasks.sql");
        }
        fs::write(&config.path, "database_path = [").unwrap();
        assert!(config.save_database_path("new.sql").is_err());
        assert_eq!(
            fs::read_to_string(&config.path).unwrap(),
            "database_path = ["
        );
        assert!(config.reload().is_err());
        fs::remove_file(&config.path).unwrap();
        fs::create_dir(&config.path).unwrap();
        assert!(config.save_database_path("new.sql").is_err());
        assert_eq!(config.database_path, "tasks.sql");
        assert_eq!(config.active_database, database);
        assert_eq!(fs::read_dir(&folder).unwrap().count(), 1);
        fs::remove_dir_all(folder).unwrap();
    }
}
