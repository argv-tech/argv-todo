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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    #[default]
    All,
    Active,
    Done,
}

impl Filter {
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Done => "Done",
        }
    }
    fn shift(self, direction: isize, count: usize) -> Self {
        let index = match self {
            Self::All => 0,
            Self::Active => 1,
            Self::Done => 2,
        };
        [Self::All, Self::Active, Self::Done]
            [(index + direction * count.min(2) as isize).clamp(0, 2) as usize]
    }
}

pub struct App {
    database: Database,
    pub todos: Vec<Todo>,
    pub filter: Filter,
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
            filter: Filter::All,
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
                (match self.filter {
                    Filter::All => true,
                    Filter::Active => !todo.done,
                    Filter::Done => todo.done,
                }) && (query.is_empty() || todo.title.to_lowercase().contains(&query))
            })
            .map(|(index, _)| index)
            .collect()
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
                VimAction::Move(Motion::Down, count) => {
                    self.help_scroll = self.help_scroll.saturating_add(count as u16).min(20);
                }
                VimAction::Move(Motion::Up, count) => {
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
                            self.database.add(&text)?
                        };
                        let was_edit = self.editing_id.is_some();
                        if !was_edit {
                            if self.filter == Filter::Done {
                                self.filter = Filter::Active;
                            }
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
                Motion::Left | Motion::Right => {
                    self.filter = self
                        .filter
                        .shift(if motion == Motion::Left { -1 } else { 1 }, count);
                    self.list.select(None);
                    self.normalize_selection();
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
                self.editing_id = None;
                self.editor = Editor::default();
                self.vim.set_mode(VimMode::Insert);
                self.message("Enter saves your task. Esc cancels.");
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
                    self.database.delete(&deleted)?;
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
                self.message("Type to filter. Enter applies. Esc cancels.");
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
    fn keyboard_workflow_add_edit_toggle_filter_delete_undo() {
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
        keys(&mut app, "xll");
        assert_eq!(app.filter, Filter::Done);
        assert_eq!(app.selected_todo().unwrap().title, "Read Rust");
        keys(&mut app, "dd");
        assert_eq!(app.visible_indices().len(), 0);
        keys(&mut app, "u");
        assert_eq!(app.selected_todo().unwrap().title, "Read Rust");
        assert!(app.selected_todo().unwrap().done);
        keys(&mut app, "hh");
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
}
