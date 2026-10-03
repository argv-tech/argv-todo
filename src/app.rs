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

pub struct App {
    database: Database,
    pub todos: Vec<Todo>,
    pub parent_id: Option<i64>,
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
            parent_id: None,
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

    pub fn visible_indices(&self) -> Vec<usize> {
        let query = if self.vim.mode() == VimMode::Search {
            self.editor.text()
        } else {
            &self.query
        };
        let query = query.to_lowercase();
        self.todos
            .iter()
            .enumerate()
            .filter(|(_, todo)| {
                todo.parent_id == self.parent_id
                    && (query.is_empty() || todo.title.to_lowercase().contains(&query))
            })
            .map(|(index, _)| index)
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

    pub fn breadcrumb(&self) -> String {
        let mut titles = Vec::new();
        let mut parent = self.parent_id;
        while let Some(id) = parent {
            let Some(todo) = self.todos.iter().find(|todo| todo.id == id) else {
                break;
            };
            titles.push(todo.title.as_str());
            parent = todo.parent_id;
            if titles.len() >= self.todos.len() {
                break;
            }
        }
        titles.reverse();
        format!("tasks / {}", titles.join(" / "))
    }

    fn begin_add(&mut self) {
        self.editing_id = None;
        self.editor = Editor::default();
        self.vim.set_mode(VimMode::Insert);
        self.message(if self.parent_id.is_some() {
            "New child task. Enter saves. Esc cancels."
        } else {
            "Enter saves your task. Esc cancels."
        });
    }

    fn enter_children(&mut self) {
        let Some(id) = self.selected_todo().map(|todo| todo.id) else {
            self.begin_add();
            return;
        };
        let has_children = self.child_counts(id).1 > 0;
        self.parent_id = Some(id);
        self.query.clear();
        self.list.select(None);
        self.normalize_selection();
        if has_children {
            self.message("Child tasks. h returns to parent. i adds a child.");
        } else {
            self.begin_add();
        }
    }

    fn leave_parent(&mut self) -> bool {
        let Some(id) = self.parent_id else {
            return false;
        };
        self.parent_id = self
            .todos
            .iter()
            .find(|todo| todo.id == id)
            .and_then(|todo| todo.parent_id);
        self.query.clear();
        let row = self
            .visible_indices()
            .iter()
            .position(|&index| self.todos[index].id == id);
        self.list.select(row);
        self.normalize_selection();
        self.message("Returned to parent level.");
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
        if self
            .parent_id
            .is_some_and(|id| !self.todos.iter().any(|todo| todo.id == id))
        {
            self.parent_id = None;
            self.query.clear();
            self.list.select(None);
        }
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
                            self.database.add(&text, self.parent_id)?
                        };
                        let was_edit = self.editing_id.is_some();
                        if !was_edit {
                            self.query.clear();
                        }
                        self.vim.set_mode(VimMode::Normal);
                        self.editing_id = None;
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
                        self.enter_children();
                        if self.vim.mode() != VimMode::Normal {
                            break;
                        }
                    }
                }
                Motion::Left => {
                    for _ in 0..count {
                        if !self.leave_parent() {
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
            VimAction::Add => self.begin_add(),
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
                    self.database.toggle(id)?;
                    self.reload(Some(id))?;
                    self.message(if done {
                        "Task reopened."
                    } else {
                        "Task completed."
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
                    self.parent_id = deleted[0].parent_id;
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
    fn nested_navigation_creates_children_and_undo_restores_tree() {
        let mut app = app();
        keys(&mut app, "iProject");
        enter(&mut app);
        let root = app.selected_todo().unwrap().id;
        keys(&mut app, "l");
        assert_eq!(app.parent_id, Some(root));
        assert_eq!(app.vim.mode(), VimMode::Insert);
        keys(&mut app, "Child");
        enter(&mut app);
        let child = app.selected_todo().unwrap().id;
        assert_eq!(app.selected_todo().unwrap().parent_id, Some(root));
        keys(&mut app, "lGrandchild");
        enter(&mut app);
        assert_eq!(app.parent_id, Some(child));
        assert_eq!(app.breadcrumb(), "tasks / Project / Child");
        keys(&mut app, "2h");
        assert_eq!(app.parent_id, None);
        assert_eq!(app.selected_todo().unwrap().id, root);
        assert_eq!(app.visible_indices().len(), 1);
        keys(&mut app, "l");
        assert_eq!(app.vim.mode(), VimMode::Normal);
        assert_eq!(app.selected_todo().unwrap().id, child);
        keys(&mut app, "iSibling");
        enter(&mut app);
        assert_eq!(app.visible_indices().len(), 2);
        keys(&mut app, "/grand");
        enter(&mut app);
        assert_eq!(
            app.visible_indices().len(),
            0,
            "Search stays within the current level"
        );
        app.dispatch(VimAction::Cancel);
        keys(&mut app, "lx");
        assert_eq!(app.parent_id, Some(child));
        assert!(app.selected_todo().unwrap().done);
        assert_eq!(app.child_counts(child), (1, 1));
        let expected = app.todos.clone();
        keys(&mut app, "hhdd");
        assert!(app.todos.is_empty());
        keys(&mut app, "u");
        assert_eq!(app.todos, expected);
        assert_eq!(app.parent_id, None);
        assert_eq!(app.selected_todo().unwrap().id, root);
        assert_eq!(app.child_counts(root), (0, 2));
    }

    #[test]
    fn empty_child_cancel_and_parent_removal_recover_to_root() {
        let mut app = app();
        keys(&mut app, "lRoot");
        enter(&mut app);
        let root = app.selected_todo().unwrap().clone();
        keys(&mut app, "l");
        app.dispatch(VimAction::Cancel);
        assert_eq!(app.parent_id, Some(root.id));
        assert_eq!(app.todos.len(), 1);
        keys(&mut app, "h");
        assert_eq!(app.selected_todo().unwrap().id, root.id);
        keys(&mut app, "lChild");
        enter(&mut app);
        app.database.delete(&[root]).unwrap();
        app.apply(VimAction::Refresh).unwrap();
        assert_eq!(app.parent_id, None);
        assert!(app.visible_indices().is_empty());
        keys(&mut app, "hjk");
        assert_eq!(app.list.selected(), None);
    }
}
