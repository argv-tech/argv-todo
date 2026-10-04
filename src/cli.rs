use anyhow::{Context, Result, bail};
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub database_path: PathBuf,
    pub database_override: bool,
}

pub fn options() -> Result<Option<Options>> {
    parse(std::env::args_os().skip(1))
}

fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Option<Options>> {
    let mut args = args.into_iter();
    let mut path = None;
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--help" | "-h") => {
                println!(
                    "argv-todo — a Vim-style terminal todo list\n\nUsage: argv-todo [--db PATH]\n\nDefault: platform config directory / argv-todo / db.sql\nLinux: ~/.config/argv-todo/db.sql (honors XDG_CONFIG_HOME)\nmacOS: ~/Library/Application Support/argv-todo/db.sql\nWindows: %APPDATA%\\argv-todo\\db.sql\n\nConfig: config.toml beside the default database, or beside --db PATH.\n        database_path = \"db.sql\" (relative to config); --db takes precedence.\n        Esc opens configuration; Enter/e edit · Enter save.\n        In fields: Esc enters Normal; Esc again cancels. Esc returns to tasks.\n        Saved settings apply on the next launch.\n\nKeys: j/k select · h parent · l child · i/a child · o/O below/above · e edit · t priority · Space toggle\n      dd delete · u undo deletion · / search · Esc config · ? help · q quit"
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
    let database_override = path.is_some();
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
    Ok(Some(Options {
        database_path: path,
        database_override,
    }))
}

#[cfg(test)]
#[path = "../tests/unit/cli.rs"]
mod tests;
