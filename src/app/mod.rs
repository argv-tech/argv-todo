mod actions;
mod editing;
mod events;
mod navigation;
mod settings;
mod tasks;

#[cfg(test)]
#[path = "../../tests/unit/app.rs"]
mod tests;

use anyhow::Result;
use ratatui::widgets::ListState;

use crate::{
    config::Config,
    db::{DEFAULT_PRIORITY, Database, Priority, Todo},
    vim_motion::{Editor, VimManager},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TreeRow {
    pub(crate) index: usize,
    pub(crate) depth: usize,
}

pub(crate) struct App {
    database: Database,
    pub(crate) todos: Vec<Todo>,
    pub(crate) adding_parent: Option<i64>,
    pub(crate) adding_relative: Option<(i64, bool)>,
    pub(crate) list: ListState,
    pub(crate) vim: VimManager,
    pub(crate) editor: Editor,
    pub(crate) editing_id: Option<i64>,
    pub(crate) input_priority: Priority,
    pub(crate) query: String,
    pub(crate) help: bool,
    pub(crate) help_scroll: u16,
    pub(crate) status: String,
    pub(crate) error: bool,
    pub(crate) config: Option<Config>,
    pub(crate) configuring: bool,
    undo: Vec<Vec<Todo>>,
    running: bool,
}

impl App {
    pub(crate) fn new(database: Database) -> Result<Self> {
        let todos = database.list()?;
        let mut app = Self {
            database,
            todos,
            adding_parent: None,
            adding_relative: None,
            list: ListState::default(),
            vim: VimManager::default(),
            editor: Editor::default(),
            editing_id: None,
            input_priority: DEFAULT_PRIORITY,
            query: String::new(),
            help: false,
            help_scroll: 0,
            status: String::new(),
            error: false,
            config: None,
            configuring: false,
            undo: Vec::new(),
            running: true,
        };
        app.normalize_selection();
        Ok(app)
    }

    fn message(&mut self, text: impl Into<String>) {
        self.status = text.into();
        self.error = false;
    }
}
