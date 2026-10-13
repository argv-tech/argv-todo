use crate::config::TaskView;
use anyhow::{Context, Result};
use std::collections::HashMap;

use ratatui::widgets::ListState;

use super::{ProjectState, ProjectTab};
use crate::{app::App, db::Project};

impl ProjectState {
    pub(in crate::app) fn new(projects: Vec<Project>) -> Self {
        let mut state = Self {
            tabs: Vec::new(),
            active: 0,
            picker: None,
        };
        state.refresh(projects);
        state
    }

    pub(in crate::app) fn refresh(&mut self, projects: Vec<Project>) {
        let active_id = self
            .tabs
            .get(self.active)
            .and_then(|tab| tab.project.as_ref())
            .map(|project| project.id);
        let mut lists: HashMap<_, _> = self
            .tabs
            .drain(..)
            .map(|tab| (tab.project.map(|project| project.id), tab.list))
            .collect();
        self.tabs.push(ProjectTab {
            project: None,
            list: lists.remove(&None).unwrap_or_default(),
        });
        for project in projects {
            let list = lists.remove(&Some(project.id)).unwrap_or_default();
            self.tabs.push(ProjectTab {
                project: Some(project),
                list,
            });
        }
        self.active = self
            .tabs
            .iter()
            .position(|tab| tab.project.as_ref().map(|project| project.id) == active_id)
            .unwrap_or(0);
    }
}

impl App {
    pub(in crate::app) fn open_project_view(&mut self) -> Result<()> {
        if self.task_view() != TaskView::Projects {
            let selected = self.selected_todo().map(|todo| todo.id);
            self.config
                .as_mut()
                .context("Configuration is unavailable")?
                .save_task_view(TaskView::Projects)?;
            self.reconcile_task_view(selected);
        }
        Ok(())
    }

    pub(crate) fn project_tabs(&self) -> impl ExactSizeIterator<Item = (Option<i64>, &str)> {
        self.projects.tabs.iter().map(|tab| {
            tab.project.as_ref().map_or((None, "Inbox"), |project| {
                (Some(project.id), project.name.as_str())
            })
        })
    }

    pub(crate) fn active_project_index(&self) -> usize {
        self.projects.active
    }

    pub(crate) fn active_project_id(&self) -> Option<i64> {
        self.projects.tabs[self.projects.active]
            .project
            .as_ref()
            .map(|project| project.id)
    }

    pub(in crate::app) fn project_list(&self) -> &ListState {
        &self.projects.tabs[self.projects.active].list
    }

    pub(in crate::app) fn project_list_mut(&mut self) -> &mut ListState {
        &mut self.projects.tabs[self.projects.active].list
    }

    pub(in crate::app) fn activate_project(&mut self, project_id: Option<i64>) {
        let index = self.project_tabs().position(|(id, _)| id == project_id);
        if let Some(index) = index {
            self.projects.active = index;
        }
    }

    pub(in crate::app) fn switch_project(&mut self, forward: bool) {
        let len = self.projects.tabs.len();
        self.projects.active = if forward {
            (self.projects.active + 1) % len
        } else {
            (self.projects.active + len - 1) % len
        };
        self.normalize_selection();
        self.message("");
    }
}
