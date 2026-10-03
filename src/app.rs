use std::collections::{HashMap, HashSet};

use anyhow::Result;
use crossterm::{
    cursor::SetCursorStyle,
    event::{self, Event},
    execute,
};
use ratatui::{DefaultTerminal, widgets::ListState};

use crate::{
    db::{Database, Todo},
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
    pub list: ListState,
    pub vim: VimManager,
    pub editor: Editor,
    pub editing_id: Option<i64>,
    pub query: String,
    pub help: bool,
    pub help_scroll: u16,
    pub status: String,
    pub error: bool,
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
            list: ListState::default(),
            vim: VimManager::default(),
            editor: Editor::default(),
            editing_id: None,
            query: String::new(),
            help: false,
            help_scroll: 0,
            status: String::new(),
            error: false,
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
        self.adding_parent = parent_id;
        self.editing_id = None;
        self.editor = Editor::default();
        self.vim.set_mode(VimMode::Insert);
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

    fn select_child(&mut self) {
        let Some(id) = self.selected_todo().map(|todo| todo.id) else {
            self.begin_add(None);
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
        } else {
            self.begin_add(Some(id));
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
        if self.vim.mode() != VimMode::Normal {
            match action {
                VimAction::Cancel => {
                    self.vim.set_mode(VimMode::Normal);
                    self.editor = Editor::default();
                    self.editing_id = None;
                    self.adding_parent = None;
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
                            self.database.rename(id, &text)?;
                            id
                        } else {
                            self.database.add(&text, self.adding_parent)?
                        };
                        let was_edit = self.editing_id.is_some();
                        if !was_edit {
                            self.query.clear();
                        }
                        self.vim.set_mode(VimMode::Normal);
                        self.editing_id = None;
                        self.adding_parent = None;
                        self.editor = Editor::default();
                        self.reload(Some(id))?;
                        self.message(if was_edit {
                            "Task updated."
                        } else {
                            "Task added."
                        });
                    }
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
            VimAction::Add => {
                let parent = self.selected_todo().and_then(|todo| todo.parent_id);
                self.begin_add(parent);
            }
            VimAction::Edit => {
                if let Some(todo) = self.selected_todo() {
                    let id = todo.id;
                    self.editor = Editor::new(todo.title.clone());
                    self.editing_id = Some(id);
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
                self.query.clear();
                self.normalize_selection();
                self.message("Search cleared.");
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
    #[test]
    fn keyboard_workflow_keeps_completed_tasks_in_one_list() {
        let mut app = app();
        keys(&mut app, "iRead a book");
        enter(&mut app);
        keys(&mut app, "iShip the app");
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
        keys(&mut app, "jkGggdd");
        assert_eq!(app.list.selected(), None);
        keys(&mut app, "i  ");
        enter(&mut app);
        assert!(app.error);
        assert_eq!(app.vim.mode(), VimMode::Insert);
        app.dispatch(VimAction::Cancel);
        keys(&mut app, "iBuy coffee");
        enter(&mut app);
        keys(&mut app, "iRead Rust");
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
        keys(&mut app, "l");
        assert_eq!(app.adding_parent, Some(root));
        assert_eq!(app.visible_indices().len(), 1);
        keys(&mut app, "Child");
        enter(&mut app);
        let child = app.selected_todo().unwrap().id;
        keys(&mut app, "lGrandchild");
        enter(&mut app);
        let grandchild = app.selected_todo().unwrap().id;
        keys(&mut app, "iSibling");
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
        keys(&mut app, "iOther root");
        enter(&mut app);
        keys(&mut app, "kl");
        app.dispatch(VimAction::Cancel);
        assert_eq!(app.selected_todo().unwrap().id, root.id);
        assert_eq!(app.adding_parent, None);
        keys(&mut app, "lChild");
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
        keys(&mut app, "jh");
        assert_eq!(app.selected_todo().unwrap().title, "Other root");
        app.database.delete(&[root]).unwrap();
        app.apply(VimAction::Refresh).unwrap();
        assert_eq!(app.selected_todo().unwrap().title, "Other root");
        assert_eq!(app.visible_indices().len(), 1);
    }
}
