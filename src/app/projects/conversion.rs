use anyhow::{Result, ensure};

use crate::app::App;

impl App {
    pub(in crate::app) fn convert_selected_project(&mut self) -> Result<()> {
        let Some(task) = self.selected_todo() else {
            self.message("Select a top-level task to convert to a project.");
            return Ok(());
        };
        ensure!(
            task.parent_id.is_none(),
            "Only top-level tasks can become projects"
        );
        let id = task.id;
        self.open_project_view()?;
        let project_id = self.database.convert_task_to_project(id)?;
        self.query.clear();
        self.reload(None)?;
        self.activate_project(Some(project_id));
        self.normalize_selection();
        self.message("Task replaced by project. Descendants and their notes preserved.");
        Ok(())
    }
}
