use anyhow::Result;
use ratatui::widgets::ListState;

use super::ProjectPicker;
use crate::{
    app::App,
    vim_motion::{InputTarget, Motion, VimAction},
};

impl App {
    pub(in crate::app) fn begin_project(&mut self) -> Result<()> {
        self.open_project_view()?;
        self.renaming_project = None;
        self.editor.reset(String::new());
        self.vim.begin_input(InputTarget::ProjectName);
        self.message("New project. Enter saves. Esc enters Normal; Esc again cancels.");
        Ok(())
    }

    pub(in crate::app) fn begin_rename_project(&mut self) -> Result<()> {
        self.open_project_view()?;
        let Some(project) = self.projects.tabs[self.projects.active].project.clone() else {
            self.message("Inbox cannot be renamed.");
            return Ok(());
        };
        self.renaming_project = Some(project.id);
        self.editor.reset(project.name);
        self.vim.begin_input(InputTarget::ProjectName);
        self.message("Rename project. Enter saves. Esc enters Normal; Esc again cancels.");
        Ok(())
    }

    pub(in crate::app) fn delete_active_project(&mut self) -> Result<()> {
        self.open_project_view()?;
        let Some(project) = self.projects.tabs[self.projects.active].project.clone() else {
            self.message("Inbox cannot be deleted.");
            return Ok(());
        };
        let stale: Vec<i64> = self
            .todos
            .iter()
            .filter(|todo| todo.project_id == Some(project.id))
            .map(|todo| todo.id)
            .chain(
                self.database
                    .list_deleted()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|todo| todo.project_id == Some(project.id))
                    .map(|todo| todo.id),
            )
            .collect();
        let tasks = self.database.delete_project(project.id)?;
        self.discard_archive_undo(&stale);
        self.query.clear();
        self.reload(None)?;
        self.activate_project(None);
        self.normalize_selection();
        if tasks == 0 {
            self.message(format!("Deleted empty project '{}'.", project.name));
        } else {
            self.message(format!(
                "Deleted project '{}' and {tasks} task(s).",
                project.name
            ));
        }
        Ok(())
    }

    pub(in crate::app) fn save_project(&mut self) -> Result<()> {
        if let Some(id) = self.renaming_project {
            self.database.rename_project(id, self.editor.text())?;
            self.projects.refresh(self.database.projects()?);
            self.activate_project(Some(id));
            self.query.clear();
            self.vim.end_input();
            self.editor.reset(String::new());
            self.renaming_project = None;
            self.normalize_selection();
            self.message("Project renamed.");
            return Ok(());
        }
        let id = self.database.create_project(self.editor.text())?;
        self.projects.refresh(self.database.projects()?);
        self.activate_project(Some(id));
        self.query.clear();
        self.vim.end_input();
        self.editor.reset(String::new());
        self.normalize_selection();
        self.message("Project created. Press i to add its first task.");
        Ok(())
    }

    pub(crate) fn project_picker(&self) -> Option<(i64, &ListState)> {
        self.projects
            .picker
            .as_ref()
            .map(|picker| (picker.task_id, &picker.list))
    }

    pub(crate) fn project_picker_list_mut(&mut self) -> Option<&mut ListState> {
        self.projects.picker.as_mut().map(|picker| &mut picker.list)
    }

    pub(in crate::app) fn begin_project_move(&mut self) {
        let Some((task_id, project_id)) =
            self.selected_todo().map(|todo| (todo.id, todo.project_id))
        else {
            self.message("Select a task to move to a project.");
            return;
        };
        let index = self
            .project_tabs()
            .position(|(id, _)| id == project_id)
            .unwrap_or(0);
        let mut list = ListState::default();
        list.select(Some(index));
        self.projects.picker = Some(ProjectPicker { task_id, list });
        self.vim.end_input();
        self.message("");
    }

    pub(in crate::app) fn apply_project_picker(&mut self, action: VimAction) -> Result<()> {
        let len = self.projects.tabs.len();
        let Some(picker) = self.projects.picker.as_mut() else {
            return Ok(());
        };
        let selected = picker.list.selected().unwrap_or(0).min(len - 1);
        match action {
            VimAction::Move(Motion::Up, count) => {
                picker.list.select(Some(selected.saturating_sub(count)))
            }
            VimAction::Move(Motion::Down, count) => picker
                .list
                .select(Some(selected.saturating_add(count).min(len - 1))),
            VimAction::FileStart => picker.list.select(Some(0)),
            VimAction::FileEnd => picker.list.select(Some(len - 1)),
            VimAction::Cancel => {
                self.projects.picker = None;
                self.vim.end_input();
                self.message("Cancelled.");
            }
            VimAction::Submit => {
                let id = picker.task_id;
                let project_id = self.projects.tabs[selected]
                    .project
                    .as_ref()
                    .map(|project| project.id);
                self.database.assign_project(id, project_id)?;
                self.projects.picker = None;
                self.vim.end_input();
                self.query.clear();
                self.reload(Some(id))?;
                self.message("Task tree moved to project.");
            }
            _ => {}
        }
        Ok(())
    }
}
