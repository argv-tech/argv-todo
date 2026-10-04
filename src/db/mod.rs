mod model;
mod queries;
mod schema;
mod trees;

#[cfg(test)]
#[path = "../../tests/unit/db.rs"]
mod tests;

pub(crate) use model::{DEFAULT_PRIORITY, Priority, Todo};

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;

pub(crate) struct Database {
    connection: Connection,
}

impl Database {
    pub(crate) fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Could not create {}", parent.display()))?;
        }
        Self::initialize(Connection::open(path)?)
    }
}
