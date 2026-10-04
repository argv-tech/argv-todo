mod actions;
mod editing;
mod events;
mod folding;
mod help;
mod navigation;
mod ordering;
mod preferences;
mod settings;
mod tasks;
mod views;

pub(crate) use settings::ConfigSetting;
pub(crate) use views::TaskPane;

#[cfg(test)]
#[path = "../../tests/unit/app.rs"]
mod tests;

use anyhow::Result;
use help::HelpView;
use ratatui::widgets::ListState;
use std::collections::HashSet;
use views::TaskPanes;

use crate::{
    config::Config,
    db::{DEFAULT_PRIORITY, Database, Priority, Todo},
    vim_motion::{Editor, VimManager},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TreeRow {
    pub(crate) index: usize,
    pub(crate) depth: usize,
    pub(crate) ghost: bool,
}

pub(crate) struct App {
    database: Database,
    pub(crate) todos: Vec<Todo>,
    pub(crate) adding_parent: Option<i64>,
    pub(crate) adding_relative: Option<(i64, bool)>,
    pub(crate) list: ListState,
    panes: TaskPanes,
    collapsed: HashSet<i64>,
    pub(crate) vim: VimManager,
    pub(crate) editor: Editor,
    pub(crate) editing_id: Option<i64>,
    pub(crate) input_priority: Priority,
    pub(crate) query: String,
    pub(crate) help: bool,
    pub(crate) help_view: HelpView,
    pub(crate) status: String,
    pub(crate) error: bool,
    pub(crate) config: Option<Config>,
    pub(crate) configuring: bool,
    pub(crate) config_setting: ConfigSetting,
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
            panes: TaskPanes::default(),
            collapsed: HashSet::new(),
            vim: VimManager::default(),
            editor: Editor::default(),
            editing_id: None,
            input_priority: DEFAULT_PRIORITY,
            query: String::new(),
            help: false,
            help_view: HelpView::default(),
            status: String::new(),
            error: false,
            config: None,
            configuring: false,
            config_setting: ConfigSetting::default(),
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
