use anyhow::{Context, Result, bail};
use std::{ffi::OsString, path::PathBuf};

pub fn database_path() -> Result<Option<PathBuf>> {
    parse(std::env::args_os().skip(1))
}

fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Option<PathBuf>> {
    let mut args = args.into_iter();
    let mut path = None;
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--help" | "-h") => {
                println!(
                    "argv-todo — a Vim-style terminal todo list\n\nUsage: argv-todo [--db PATH]\n\nDefault: platform config directory / argv-todo / db.sql\nLinux: ~/.config/argv-todo/db.sql (honors XDG_CONFIG_HOME)\nmacOS: ~/Library/Application Support/argv-todo/db.sql\nWindows: %APPDATA%\\argv-todo\\db.sql\n\nKeys: j/k select · h parent · l child · i/a child · o/O below/above · e edit · t priority · Space toggle\n      dd delete · u undo deletion · / search · ? help · q quit"
                );
                return Ok(None);
            }
            Some("--db") => {
                path = Some(PathBuf::from(
                    args.next().context("--db requires a file path")?,
                ))
            }
            Some("--version" | "-V") => {
                println!("argv-todo {}", env!("CARGO_PKG_VERSION"));
                return Ok(None);
            }
            _ => bail!(
                "Unknown argument: {}. Use --help for usage.",
                arg.to_string_lossy()
            ),
        }
    }
    let path = match path {
        Some(path) => path,
        None => dirs::config_dir()
            .context("Could not determine your config directory; use --db PATH")?
            .join("argv-todo")
            .join("db.sql"),
    };
    anyhow::ensure!(
        !path.as_os_str().is_empty(),
        "Database path cannot be empty"
    );
    Ok(Some(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_uses_platform_config_directory() {
        assert_eq!(
            parse([]).unwrap().unwrap(),
            dirs::config_dir().unwrap().join("argv-todo/db.sql")
        );
    }
    #[test]
    fn accepts_override_and_rejects_missing_or_unknown_arguments() {
        assert_eq!(
            parse(["--db".into(), "/tmp/custom.sql".into()]).unwrap(),
            Some(PathBuf::from("/tmp/custom.sql"))
        );
        assert!(parse(["--db".into()]).is_err());
        assert!(parse(["--unknown".into()]).is_err());
    }
}
