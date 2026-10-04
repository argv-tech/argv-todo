use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use crossterm::{
    cursor::SetCursorStyle,
    event::{self, Event},
    execute,
};
use ratatui::{DefaultTerminal, widgets::ListState};

use crate::{
    config::Config,
    db::{DEFAULT_PRIORITY, Database, Priority, Todo},
    vim_motion::{
        editor::Editor,
        manager::VimManager,
        vim_action::{Motion, VimAction},
        vim_mode::VimMode,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeRow {
    pub index: usize,
    pub depth: usize,
}

pub struct App {
    database: Database,
    pub todos: Vec<Todo>,
    pub adding_parent: Option<i64>,
    pub adding_relative: Option<(i64, bool)>,
    pub list: ListState,
    pub vim: VimManager,
    pub editor: Editor,
    pub editing_id: Option<i64>,
    pub input_priority: Priority,
    pub query: String,
    pub help: bool,
    pub help_scroll: u16,
    pub status: String,
    pub error: bool,
    pub config: Option<Config>,
    pub configuring: bool,
    undo: Vec<Vec<Todo>>,
    running: bool,
}

impl App {
    pub fn new(database: Database) -> Result<Self> {
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

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.running {
            let shape = if self.vim.mode() == VimMode::Normal {
                SetCursorStyle::SteadyBlock
            } else {
                SetCursorStyle::SteadyBar
            };
            execute!(std::io::stdout(), shape)?;
            terminal.draw(|frame| crate::ui::draw(frame, self))?;
            match event::read()? {
                Event::Key(key) => {
                    if let Some(action) = self.vim.handle(key) {
                        self.dispatch(action);
                    }
                }
                Event::Paste(text) if self.vim.mode() != VimMode::Normal && !self.help => {
                    self.editor.insert(&text);
                    self.normalize_selection();
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// All tasks in tree order. Searches retain the ancestors of matching tasks.
    pub fn visible_rows(&self) -> Vec<TreeRow> {
        let query = if self.vim.mode() == VimMode::Search {
            self.editor.text()
        } else {
            &self.query
        }
        .to_lowercase();
        let by_id: HashMap<_, _> = self
            .todos
            .iter()
            .enumerate()
            .map(|(index, todo)| (todo.id, index))
            .collect();
        let mut children: HashMap<Option<i64>, Vec<usize>> = HashMap::new();
        let mut included = HashSet::new();
        for (index, todo) in self.todos.iter().enumerate() {
            let parent = todo.parent_id.filter(|id| by_id.contains_key(id));
            children.entry(parent).or_default().push(index);
            if query.is_empty() || todo.title.to_lowercase().contains(&query) {
                let mut ancestor = Some(index);
                while let Some(index) = ancestor {
                    if !included.insert(index) {
                        break;
                    }
                    ancestor = self.todos[index]
                        .parent_id
                        .and_then(|id| by_id.get(&id).copied());
                }
            }
        }
        let mut stack: Vec<_> = children
            .get(&None)
            .into_iter()
            .flatten()
            .rev()
            .map(|&index| TreeRow { index, depth: 0 })
            .collect();
        let mut rows = Vec::new();
        while let Some(row) = stack.pop() {
            if !included.contains(&row.index) {
                continue;
            }
            if let Some(children) = children.get(&Some(self.todos[row.index].id)) {
                stack.extend(children.iter().rev().map(|&index| TreeRow {
                    index,
                    depth: row.depth + 1,
                }));
            }
            rows.push(row);
        }
        rows
    }

    pub fn visible_indices(&self) -> Vec<usize> {
        self.visible_rows()
            .into_iter()
            .map(|row| row.index)
            .collect()
    }

    pub fn child_counts(&self, id: i64) -> (usize, usize) {
        self.todos
            .iter()
            .filter(|todo| todo.parent_id == Some(id))
            .fold((0, 0), |(done, total), todo| {
                (done + usize::from(todo.done), total + 1)
            })
    }

    fn begin_add(&mut self, parent_id: Option<i64>) {
        let selected_id = self.selected_todo().map(|todo| todo.id);
        // Show the full tree so the draft and saved task occupy the same place.
        self.query.clear();
        self.adding_parent = parent_id;
        self.adding_relative = None;
        self.editing_id = None;
        self.input_priority = parent_id
            .and_then(|id| self.todos.iter().find(|todo| todo.id == id))
            .map_or(DEFAULT_PRIORITY, |todo| todo.priority);
        self.editor = Editor::default();
        self.vim.set_mode(VimMode::Insert);
        if let Some(id) = selected_id {
            self.select_id(id);
        }
        self.message(if parent_id.is_some() {
            "New child task. Enter saves. Esc cancels."
        } else {
            "Enter saves your task. Esc cancels."
        });
    }

    fn select_id(&mut self, id: i64) {
        self.list.select(
            self.visible_indices()
                .iter()
                .position(|&index| self.todos[index].id == id),
        );
        self.normalize_selection();
    }

    fn begin_sibling(&mut self, above: bool) {
        let selected = self
            .selected_todo()
            .map(|todo| (todo.id, todo.parent_id, todo.priority));
        self.begin_add(selected.and_then(|(_, parent, _)| parent));
        if let Some((id, _, priority)) = selected {
            self.adding_relative = Some((id, above));
            self.input_priority = priority;
        }
    }

    fn set_priority(&mut self, priority: Priority) -> Result<()> {
        if let Some(id) = self.selected_todo().map(|todo| todo.id) {
            self.database.set_priority(id, priority)?;
            self.reload(Some(id))?;
            self.message(format!("Priority {}.", priority.label()));
        }
        Ok(())
    }

    fn select_child(&mut self) {
        let Some(id) = self.selected_todo().map(|todo| todo.id) else {
            return;
        };
        if let Some(child) = self
            .todos
            .iter()
            .find(|todo| todo.parent_id == Some(id))
            .map(|todo| todo.id)
        {
            self.query.clear();
            self.select_id(child);
            self.message("Child selected.");
        }
    }

    fn select_parent(&mut self) -> bool {
        let Some(id) = self.selected_todo().and_then(|todo| todo.parent_id) else {
            return false;
        };
        self.query.clear();
        self.select_id(id);
        self.message("Parent selected.");
        true
    }

    pub fn selected_todo(&self) -> Option<&Todo> {
        self.list
            .selected()
            .and_then(|row| self.visible_indices().get(row).copied())
            .map(|index| &self.todos[index])
    }

    fn normalize_selection(&mut self) {
        let len = self.visible_indices().len();
        self.list.select(if len == 0 {
            None
        } else {
            Some(self.list.selected().unwrap_or(0).min(len - 1))
        });
    }

    fn reload(&mut self, selected_id: Option<i64>) -> Result<()> {
        self.todos = self.database.list()?;
        if let Some(id) = selected_id {
            let row = self
                .visible_indices()
                .iter()
                .position(|&index| self.todos[index].id == id);
            if row.is_some() {
                self.list.select(row);
            }
        }
        self.normalize_selection();
        Ok(())
    }

    fn message(&mut self, text: impl Into<String>) {
        self.status = text.into();
        self.error = false;
    }

    fn dispatch(&mut self, action: VimAction) {
        if let Err(error) = self.apply(action) {
            self.status = format!("Error: {error:#}");
            self.error = true;
        }
    }

    fn apply(&mut self, action: VimAction) -> Result<()> {
        if self.help {
            match action {
                VimAction::Cancel | VimAction::Help => self.help = false,
                VimAction::Quit => self.running = false,
                VimAction::Move(Motion::Down | Motion::Right, count) => {
                    self.help_scroll = self.help_scroll.saturating_add(count as u16).min(20);
                }
                VimAction::Move(Motion::Up | Motion::Left, count) => {
                    self.help_scroll = self.help_scroll.saturating_sub(count as u16);
                }
                VimAction::FileStart => self.help_scroll = 0,
                _ => {}
            }
            self.vim.set_mode(VimMode::Normal);
            return Ok(());
        }
        if action == VimAction::Quit {
            self.running = false;
            return Ok(());
        }
        if self.configuring {
            return self.apply_config(action);
        }
        if self.vim.mode() != VimMode::Normal {
            match action {
                VimAction::Cancel => {
                    self.vim.set_mode(VimMode::Normal);
                    self.editor = Editor::default();
                    self.editing_id = None;
                    self.adding_parent = None;
                    self.adding_relative = None;
                    self.normalize_selection();
                    self.message("Cancelled.");
                }
                VimAction::Submit => {
                    let text = self.editor.text().trim().to_owned();
                    if self.vim.mode() == VimMode::Search {
                        self.query = text;
                        self.vim.set_mode(VimMode::Normal);
                        self.normalize_selection();
                        self.message(if self.query.is_empty() {
                            "Search cleared."
                        } else {
                            "Search applied. Esc clears it."
                        });
                    } else {
                        anyhow::ensure!(!text.is_empty(), "Task title cannot be empty");
                        let id = if let Some(id) = self.editing_id {
                            self.database.update(id, &text, self.input_priority)?;
                            id
                        } else if let Some((id, above)) = self.adding_relative {
                            self.database
                                .add_relative(&text, id, above, self.input_priority)?
                        } else {
                            self.database
                                .add(&text, self.adding_parent, self.input_priority)?
                        };
                        let was_edit = self.editing_id.is_some();
                        if !was_edit {
                            self.query.clear();
                        }
                        self.vim.set_mode(VimMode::Normal);
                        self.editing_id = None;
                        self.adding_parent = None;
                        self.adding_relative = None;
                        self.editor = Editor::default();
                        self.reload(Some(id))?;
                        self.message(if was_edit {
                            "Task updated."
                        } else {
                            "Task added."
                        });
                    }
                }
                VimAction::CyclePriority if self.vim.mode() == VimMode::Insert => {
                    self.input_priority = self.input_priority.next();
                }
                _ => {
                    self.editor.apply(action);
                    if self.vim.mode() == VimMode::Search {
                        self.normalize_selection();
                    }
                }
            }
            return Ok(());
        }
        match action {
            VimAction::Move(motion, count) => match motion {
                Motion::Up | Motion::Down => {
                    let len = self.visible_indices().len();
                    if len > 0 {
                        let selected = self.list.selected().unwrap_or(0);
                        self.list.select(Some(if motion == Motion::Down {
                            selected.saturating_add(count).min(len - 1)
                        } else {
                            selected.saturating_sub(count)
                        }));
                    }
                }
                Motion::Right => {
                    for _ in 0..count {
                        self.select_child();
                        if self.vim.mode() != VimMode::Normal {
                            break;
                        }
                    }
                }
                Motion::Left => {
                    for _ in 0..count {
                        if !self.select_parent() {
                            break;
                        }
                    }
                }
                _ => {}
            },
            VimAction::FileStart => {
                self.list.select(Some(0));
                self.normalize_selection();
            }
            VimAction::FileEnd => {
                self.list
                    .select(self.visible_indices().len().checked_sub(1));
            }
            VimAction::AddChild => {
                let parent = self.selected_todo().map(|todo| todo.id);
                self.begin_add(parent);
            }
            VimAction::AddBelow => self.begin_sibling(false),
            VimAction::AddAbove => self.begin_sibling(true),
            VimAction::Edit => {
                if let Some(todo) = self.selected_todo() {
                    let id = todo.id;
                    let priority = todo.priority;
                    self.editor = Editor::new(todo.title.clone());
                    self.editing_id = Some(id);
                    self.input_priority = priority;
                    self.vim.set_mode(VimMode::Insert);
                    self.message("Editing task. Enter saves. Esc cancels.");
                } else {
                    self.message("Select a task to edit.");
                }
            }
            VimAction::Toggle => {
                if let Some(todo) = self.selected_todo() {
                    let id = todo.id;
                    let done = todo.done;
                    let changed = self.database.toggle(id)?;
                    self.reload(Some(id))?;
                    let action = if done { "Reopened" } else { "Completed" };
                    self.message(if changed == 1 {
                        format!("{action} task.")
                    } else {
                        format!("{action} task and {} descendants.", changed - 1)
                    });
                }
            }
            VimAction::Priority(priority) => self.set_priority(priority)?,
            VimAction::CyclePriority => {
                if let Some(priority) = self.selected_todo().map(|todo| todo.priority) {
                    self.set_priority(priority.next())?;
                }
            }
            VimAction::Delete(count) => {
                let visible = self.visible_indices();
                let start = self.list.selected().unwrap_or(0);
                let deleted: Vec<_> = visible
                    .into_iter()
                    .skip(start)
                    .take(count)
                    .map(|i| self.todos[i].clone())
                    .collect();
                if !deleted.is_empty() {
                    let deleted = self.database.delete(&deleted)?;
                    if deleted.is_empty() {
                        self.reload(None)?;
                        self.message("Task no longer exists.");
                        return Ok(());
                    }
                    let len = deleted.len();
                    self.undo.push(deleted);
                    self.reload(None)?;
                    self.message(format!("Deleted {len} task(s). Press u to undo."));
                }
            }
            VimAction::Undo => {
                if let Some(deleted) = self.undo.last() {
                    self.database.restore(deleted)?;
                    let id = deleted[0].id;
                    self.query.clear();
                    self.undo.pop();
                    self.reload(Some(id))?;
                    self.message("Deletion undone.");
                } else {
                    self.message("No deletion to undo in this session.");
                }
            }
            VimAction::Search => {
                self.editor = Editor::new(self.query.clone());
                self.vim.set_mode(VimMode::Search);
                self.message("Type to search. Enter applies. Esc cancels.");
            }
            VimAction::Cancel => {
                if self.query.is_empty() {
                    self.configuring = true;
                    self.vim.set_mode(VimMode::Normal);
                    self.message("");
                    if let Some(config) = &mut self.config {
                        config.reload()?;
                    }
                } else {
                    self.query.clear();
                    self.normalize_selection();
                    self.message("Search cleared.");
                }
            }
            VimAction::Help => {
                self.help = true;
                self.help_scroll = 0;
            }
            VimAction::Refresh => {
                let id = self.selected_todo().map(|todo| todo.id);
                self.reload(id)?;
                self.message("Reloaded tasks from SQLite.");
            }
            _ => {}
        }
        Ok(())
    }

    fn apply_config(&mut self, action: VimAction) -> Result<()> {
        if self.vim.mode() != VimMode::Normal {
            match action {
                VimAction::Cancel => {
                    self.vim.set_mode(VimMode::Normal);
                    self.editor = Editor::default();
                    self.message("");
                }
                VimAction::Submit => {
                    self.config
                        .as_mut()
                        .context("Configuration is unavailable")?
                        .save_database_path(self.editor.text())?;
                    self.vim.set_mode(VimMode::Normal);
                    self.editor = Editor::default();
                    self.message("Saved. Applies on next launch.");
                }
                _ => self.editor.apply(action),
            }
        } else {
            self.vim.set_mode(VimMode::Normal);
            match action {
                VimAction::Cancel => {
                    self.configuring = false;
                    self.message("");
                }
                VimAction::Edit | VimAction::AddChild | VimAction::Toggle => {
                    let config = self
                        .config
                        .as_ref()
                        .context("Configuration is unavailable")?;
                    self.editor = Editor::new(config.database_path.clone());
                    self.vim.set_mode(VimMode::Insert);
                    self.message("");
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::path::PathBuf;
    fn app() -> App {
        App::new(Database::memory()).unwrap()
    }
    fn keys(app: &mut App, text: &str) {
        for c in text.chars() {
            if let Some(action) = app
                .vim
                .handle(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
            {
                app.dispatch(action);
            }
        }
    }
    fn enter(app: &mut App) {
        let action = app
            .vim
            .handle(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
            .unwrap();
        app.dispatch(action);
    }

    fn cycle_input_priority(app: &mut App) {
        let action = app
            .vim
            .handle(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL))
            .unwrap();
        app.dispatch(action);
    }

    fn escape(app: &mut App) {
        let action = app
            .vim
            .handle(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
            .unwrap();
        app.dispatch(action);
    }

    #[test]
    fn escape_opens_config_and_edits_settings_without_changing_tasks() {
        let folder = std::env::temp_dir().join(format!(
            "argv-todo-config-ui-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let config = Config::load(&folder.join("custom.sql"), true).unwrap();
        let path = config.path.clone();
        let mut app = app();
        app.config = Some(config);
        keys(&mut app, "iKeep this task");
        enter(&mut app);
        let tasks = app.todos.clone();
        let selection = app.list.selected();

        // Input, help and applied searches keep their existing Escape behavior.
        keys(&mut app, "eChanged task");
        escape(&mut app);
        assert!(!app.configuring);
        assert_eq!(app.todos, tasks);
        keys(&mut app, "?");
        escape(&mut app);
        assert!(!app.help && !app.configuring);
        keys(&mut app, "/Keep");
        enter(&mut app);
        escape(&mut app);
        assert!(app.query.is_empty() && !app.configuring);

        escape(&mut app);
        assert!(app.configuring);
        keys(&mut app, "ddtu");
        assert_eq!(app.todos, tasks);
        enter(&mut app);
        assert_eq!(app.vim.mode(), VimMode::Insert);
        app.dispatch(VimAction::Clear);
        enter(&mut app);
        assert!(app.error && app.configuring);
        assert_eq!(app.vim.mode(), VimMode::Insert);
        assert_eq!(parse_config(&path), PathBuf::from("db.sql"));
        keys(&mut app, "storage/tasks.sql");
        enter(&mut app);
        assert!(!app.error && app.configuring);
        assert_eq!(app.vim.mode(), VimMode::Normal);
        assert_eq!(parse_config(&path), PathBuf::from("storage/tasks.sql"));
        assert_eq!(
            app.config.as_ref().unwrap().active_database,
            folder.join("custom.sql")
        );
        assert!(!folder.join("storage/tasks.sql").exists());

        keys(&mut app, "eDiscard this change");
        escape(&mut app);
        assert!(app.configuring);
        assert_eq!(parse_config(&path), PathBuf::from("storage/tasks.sql"));
        escape(&mut app);
        assert!(!app.configuring);
        assert_eq!(app.todos, tasks);
        assert_eq!(app.list.selected(), selection);

        std::fs::write(&path, "database_path = 'external.sql'").unwrap();
        escape(&mut app);
        assert_eq!(app.config.as_ref().unwrap().database_path, "external.sql");
        keys(&mut app, "q");
        assert!(!app.running);
        std::fs::remove_dir_all(folder).unwrap();

        fn parse_config(path: &std::path::Path) -> std::path::PathBuf {
            let content = std::fs::read_to_string(path).unwrap();
            let value = content.parse::<toml::Table>().unwrap();
            value["database_path"].as_str().unwrap().into()
        }
    }

    #[test]
    fn new_children_inherit_parent_priority_for_all_creation_keys() {
        for priority in [Priority::High, Priority::Mid, Priority::Low] {
            for key in ["a", "i"] {
                let database = Database::memory();
                let parent = database.add("Parent", None, priority).unwrap();
                database
                    .add("Existing child", Some(parent), Priority::Low)
                    .unwrap();
                let mut app = App::new(database).unwrap();
                keys(&mut app, key);
                assert_eq!(app.vim.mode(), VimMode::Insert);
                assert_eq!(app.adding_parent, Some(parent));
                assert_eq!(app.input_priority, priority);
                keys(&mut app, "New child");
                enter(&mut app);
                assert_eq!(app.selected_todo().unwrap().parent_id, Some(parent));
                assert_eq!(app.selected_todo().unwrap().priority, priority);
            }
        }
    }

    #[test]
    fn creation_keys_place_children_and_siblings_and_undo_keeps_order() {
        let mut app = app();
        for key in ["a", "i", "o", "O"] {
            keys(&mut app, key);
            assert_eq!(app.vim.mode(), VimMode::Insert);
            assert_eq!(app.adding_parent, None);
            assert_eq!(app.adding_relative, None);
            app.dispatch(VimAction::Cancel);
        }
        keys(&mut app, "iRoot");
        enter(&mut app);
        let root = app.selected_todo().unwrap().id;
        keys(&mut app, "aFirst child");
        assert_eq!(app.adding_parent, Some(root));
        enter(&mut app);
        let first = app.selected_todo().unwrap().id;
        keys(&mut app, "iGrandchild");
        assert_eq!(app.adding_parent, Some(first));
        enter(&mut app);
        keys(&mut app, "hoSecond child");
        enter(&mut app);
        keys(&mut app, "OBetween children");
        enter(&mut app);
        assert_eq!(app.selected_todo().unwrap().parent_id, Some(root));
        keys(&mut app, "ggoBelow root");
        enter(&mut app);
        app.select_id(root);
        keys(&mut app, "OAbove root");
        enter(&mut app);
        let selected = app.selected_todo().unwrap().id;
        app.reload(Some(selected)).unwrap();
        assert_eq!(
            app.visible_rows()
                .iter()
                .map(|row| { (app.todos[row.index].title.as_str(), row.depth) })
                .collect::<Vec<_>>(),
            [
                ("Above root", 0),
                ("Root", 0),
                ("First child", 1),
                ("Grandchild", 2),
                ("Between children", 1),
                ("Second child", 1),
                ("Below root", 0),
            ]
        );
        let expected = app.todos.clone();
        app.select_id(root);
        keys(&mut app, "O");
        app.dispatch(VimAction::Cancel);
        assert_eq!(app.selected_todo().unwrap().id, root);
        assert_eq!(app.todos, expected);
        keys(&mut app, "ddu");
        assert_eq!(app.todos, expected);
        keys(&mut app, "pho");
        assert_eq!(app.input_priority, Priority::High);
        keys(&mut app, "High sibling");
        enter(&mut app);
        assert_eq!(app.selected_todo().unwrap().priority, Priority::High);
        assert_eq!(app.selected_todo().unwrap().parent_id, None);
    }

    #[test]
    fn priorities_sort_siblings_keep_selection_and_survive_edit_and_undo() {
        let mut app = app();
        keys(&mut app, "iFirst root");
        enter(&mut app);
        let first = app.selected_todo().unwrap().id;
        keys(&mut app, "oSecond root");
        enter(&mut app);
        let second = app.selected_todo().unwrap().id;
        for (priority, order) in [
            (Priority::High, [second, first]),
            (Priority::Low, [first, second]),
            (Priority::Mid, [first, second]),
        ] {
            keys(&mut app, "t");
            assert_eq!(app.selected_todo().unwrap().id, second);
            assert_eq!(app.selected_todo().unwrap().priority, priority);
            assert_eq!(
                app.visible_indices()
                    .iter()
                    .map(|&i| app.todos[i].id)
                    .collect::<Vec<_>>(),
                order
            );
        }
        keys(&mut app, "phiMid child");
        enter(&mut app);
        keys(&mut app, "pm");
        let mid_child = app.selected_todo().unwrap().id;
        keys(&mut app, "oHigh child");
        cycle_input_priority(&mut app);
        enter(&mut app);
        let high_child = app.selected_todo().unwrap().id;
        assert_eq!(app.selected_todo().unwrap().priority, Priority::High);
        keys(&mut app, "iGrandchild");
        enter(&mut app);
        let grandchild = app.selected_todo().unwrap().id;
        keys(&mut app, "hpl");
        let rows = app.visible_rows();
        assert_eq!(
            rows.iter()
                .map(|row| (app.todos[row.index].id, row.depth))
                .collect::<Vec<_>>(),
            [
                (second, 0),
                (mid_child, 1),
                (high_child, 1),
                (grandchild, 2),
                (first, 0),
            ]
        );
        assert_eq!(app.selected_todo().unwrap().id, high_child);
        keys(&mut app, "e");
        cycle_input_priority(&mut app);
        app.dispatch(VimAction::Clear);
        keys(&mut app, "Updated child");
        enter(&mut app);
        assert_eq!(app.selected_todo().unwrap().priority, Priority::Mid);
        assert_eq!(app.selected_todo().unwrap().title, "Updated child");
        let expected = app.todos.clone();
        keys(&mut app, "e");
        cycle_input_priority(&mut app);
        app.dispatch(VimAction::Cancel);
        assert_eq!(app.todos, expected, "Cancel must not save priority changes");
        keys(&mut app, "ggdd");
        assert_eq!(app.todos.len(), 1);
        keys(&mut app, "u");
        assert_eq!(app.todos, expected);
        assert_eq!(app.selected_todo().unwrap().id, second);
    }
    #[test]
    fn keyboard_workflow_keeps_completed_tasks_in_one_list() {
        let mut app = app();
        keys(&mut app, "iRead a book");
        enter(&mut app);
        keys(&mut app, "oShip the app");
        enter(&mut app);
        assert_eq!(app.selected_todo().unwrap().title, "Ship the app");
        keys(&mut app, "ke");
        app.dispatch(VimAction::Clear);
        keys(&mut app, "Read Rust");
        enter(&mut app);
        keys(&mut app, "x");
        assert_eq!(app.visible_indices().len(), 2);
        assert_eq!(app.selected_todo().unwrap().title, "Read Rust");
        assert!(app.selected_todo().unwrap().done);
        keys(&mut app, "j");
        assert_eq!(app.selected_todo().unwrap().title, "Ship the app");
        keys(&mut app, "kdd");
        assert_eq!(app.visible_indices().len(), 1);
        assert_eq!(app.selected_todo().unwrap().title, "Ship the app");
        keys(&mut app, "u");
        assert_eq!(app.selected_todo().unwrap().title, "Read Rust");
        assert!(app.selected_todo().unwrap().done);
        keys(&mut app, "2j");
        assert_eq!(app.selected_todo().unwrap().title, "Ship the app");
        keys(&mut app, "2k");
        assert_eq!(app.selected_todo().unwrap().title, "Read Rust");
        assert_eq!(app.todos.len(), 2);
    }
    #[test]
    fn search_cancel_blank_title_and_empty_navigation() {
        let mut app = app();
        keys(&mut app, "jkGggdd3l");
        assert_eq!(app.list.selected(), None);
        assert_eq!(app.vim.mode(), VimMode::Normal);
        keys(&mut app, "i  ");
        enter(&mut app);
        assert!(app.error);
        assert_eq!(app.vim.mode(), VimMode::Insert);
        app.dispatch(VimAction::Cancel);
        keys(&mut app, "iBuy coffee");
        enter(&mut app);
        keys(&mut app, "oRead Rust");
        enter(&mut app);
        keys(&mut app, "/COFFEE");
        assert_eq!(app.visible_indices().len(), 1);
        app.dispatch(VimAction::Cancel);
        assert_eq!(app.visible_indices().len(), 2);
        keys(&mut app, "/rust");
        enter(&mut app);
        assert_eq!(app.selected_todo().unwrap().title, "Read Rust");
        app.dispatch(VimAction::Cancel);
        keys(&mut app, "gg2dd");
        assert_eq!(app.todos.len(), 0);
        keys(&mut app, "u");
        assert_eq!(app.todos.len(), 2);
    }

    #[test]
    fn nested_tasks_stay_in_one_view_and_undo_restores_tree() {
        let mut app = app();
        keys(&mut app, "iProject");
        enter(&mut app);
        let root = app.selected_todo().unwrap().id;
        keys(&mut app, "i");
        assert_eq!(app.adding_parent, Some(root));
        assert_eq!(app.visible_indices().len(), 1);
        keys(&mut app, "Child");
        enter(&mut app);
        let child = app.selected_todo().unwrap().id;
        keys(&mut app, "iGrandchild");
        enter(&mut app);
        let grandchild = app.selected_todo().unwrap().id;
        keys(&mut app, "3l");
        assert_eq!(app.vim.mode(), VimMode::Normal);
        assert_eq!(app.selected_todo().unwrap().id, grandchild);
        assert_eq!(app.todos.len(), 3);
        keys(&mut app, "oSibling");
        enter(&mut app);
        assert_eq!(app.selected_todo().unwrap().parent_id, Some(child));
        assert_eq!(
            app.visible_rows()
                .iter()
                .map(|row| row.depth)
                .collect::<Vec<_>>(),
            [0, 1, 2, 2]
        );
        keys(&mut app, "2h");
        assert_eq!(app.selected_todo().unwrap().id, root);
        assert_eq!(app.visible_indices().len(), 4);
        keys(&mut app, "ll");
        assert_eq!(app.selected_todo().unwrap().id, grandchild);
        assert_eq!(app.visible_indices().len(), 4);
        keys(&mut app, "x");
        let expected = app.todos.clone();
        keys(&mut app, "/grand");
        enter(&mut app);
        let rows = app.visible_rows();
        assert_eq!(
            rows.iter()
                .map(|row| app.todos[row.index].id)
                .collect::<Vec<_>>(),
            [root, child, grandchild]
        );
        assert_eq!(
            rows.iter().map(|row| row.depth).collect::<Vec<_>>(),
            [0, 1, 2]
        );
        app.dispatch(VimAction::Cancel);
        keys(&mut app, "gghdd");
        assert!(app.todos.is_empty());
        keys(&mut app, "u");
        assert_eq!(app.todos, expected);
        assert_eq!(app.selected_todo().unwrap().id, root);
        assert_eq!(app.visible_indices().len(), 4);
        // Toggling a parent also reaches descendants hidden by search.
        keys(&mut app, "/Project");
        enter(&mut app);
        assert_eq!(app.visible_indices().len(), 1);
        keys(&mut app, "x");
        assert_eq!(app.selected_todo().unwrap().id, root);
        assert!(app.todos.iter().all(|todo| todo.done));
        assert!(app.database.list().unwrap().iter().all(|todo| todo.done));
        app.dispatch(VimAction::Cancel);
        keys(&mut app, "x");
        assert!(app.todos.iter().all(|todo| !todo.done));
    }

    #[test]
    fn tree_order_groups_children_and_cancel_keeps_parent_selected() {
        let mut app = app();
        keys(&mut app, "iRoot");
        enter(&mut app);
        let root = app.selected_todo().unwrap().clone();
        keys(&mut app, "oOther root");
        enter(&mut app);
        keys(&mut app, "ki");
        app.dispatch(VimAction::Cancel);
        assert_eq!(app.selected_todo().unwrap().id, root.id);
        assert_eq!(app.adding_parent, None);
        keys(&mut app, "iChild");
        enter(&mut app);
        let rows = app.visible_rows();
        assert_eq!(
            rows.iter()
                .map(|row| app.todos[row.index].title.as_str())
                .collect::<Vec<_>>(),
            ["Root", "Child", "Other root"]
        );
        assert_eq!(
            rows.iter().map(|row| row.depth).collect::<Vec<_>>(),
            [0, 1, 0]
        );
        let child = app.selected_todo().unwrap().id;
        keys(&mut app, "/Child");
        enter(&mut app);
        keys(&mut app, "i");
        assert!(app.query.is_empty());
        assert_eq!(app.adding_parent, Some(child));
        assert_eq!(app.selected_todo().unwrap().id, child);
        assert_eq!(app.todos.len(), 3, "Drafts are not saved before Enter");
        app.dispatch(VimAction::Cancel);
        assert_eq!(app.selected_todo().unwrap().id, child);
        keys(&mut app, "jh");
        assert_eq!(app.selected_todo().unwrap().title, "Other root");
        app.database.delete(&[root]).unwrap();
        app.apply(VimAction::Refresh).unwrap();
        assert_eq!(app.selected_todo().unwrap().title, "Other root");
        assert_eq!(app.visible_indices().len(), 1);
    }
}
