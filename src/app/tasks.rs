use anyhow::Result;

use super::App;
use crate::{
    db::Priority,
    vim_motion::{InputTarget, Motion, VimAction, VimMode},
};

impl App {
    fn set_priority(&mut self, priority: Priority) -> Result<()> {
        if let Some(id) = self.selected_todo().map(|todo| todo.id) {
            self.database.set_priority(id, priority)?;
            self.reload(Some(id))?;
            self.message(format!("Priority {}.", priority.label()));
        }
        Ok(())
    }

    pub(super) fn reload(&mut self, selected_id: Option<i64>) -> Result<()> {
        self.todos = self.database.list()?;
        self.collapsed
            .retain(|id| self.todos.iter().any(|todo| todo.id == *id));
        if let Some(id) = selected_id {
            self.select_id(id);
        }
        self.normalize_selection();
        Ok(())
    }

    pub(super) fn apply_tasks(&mut self, action: VimAction) -> Result<()> {
        match action {
            VimAction::ToggleCollapse => self.toggle_collapse(),
            VimAction::MoveTask(movement, count) => self.move_task(movement, count)?,
            VimAction::Move(motion, count) => match motion {
                Motion::Up | Motion::Down => {
                    let len = self.visible_indices().len();
                    if len > 0 {
                        let selected = self.active_list().selected().unwrap_or(0);
                        self.active_list_mut()
                            .select(Some(if motion == Motion::Down {
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
                self.active_list_mut().select(Some(0));
                self.normalize_selection();
            }
            VimAction::FileEnd => {
                let last = self.visible_indices().len().checked_sub(1);
                self.active_list_mut().select(last);
            }
            VimAction::SwitchPane | VimAction::SwitchPaneBackward => self.switch_task_pane(),
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
                    self.editor.reset(todo.title.clone());
                    self.editing_id = Some(id);
                    self.input_priority = priority;
                    self.vim.begin_input(InputTarget::Task);
                    self.message(
                        "Editing task. Enter saves. Esc enters Normal; Esc again cancels.",
                    );
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
                let start = self.active_list().selected().unwrap_or(0);
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
                let selected_id = self.selected_todo().map(|todo| todo.id);
                self.collapsed.clear();
                if let Some(id) = selected_id {
                    self.select_id(id);
                }
                self.editor.reset(self.query.clone());
                self.vim.begin_input(InputTarget::Search);
                self.message(
                    "Type to search. Enter applies. Esc enters Normal; Esc again cancels.",
                );
            }
            VimAction::Cancel => {
                if self.query.is_empty() {
                    self.open_config()?;
                } else {
                    self.query.clear();
                    self.normalize_selection();
                    self.message("Search cleared.");
                }
            }
            VimAction::Help => {
                self.help = true;
                self.help_view = Default::default();
            }
            VimAction::Refresh => {
                let id = self.selected_todo().map(|todo| todo.id);
                self.reload(id)?;
                self.message("Reloaded tasks from SQLite.");
            }
            _ => {}
        }
        self.normalize_selection();
        Ok(())
    }
}
