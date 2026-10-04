use anyhow::{Context, Result};

use super::App;
use crate::vim_motion::{InputTarget, Motion, VimAction, VimMode};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfigSetting {
    #[default]
    DatabasePath,
    TaskView,
}

impl ConfigSetting {
    fn other(self) -> Self {
        match self {
            Self::DatabasePath => Self::TaskView,
            Self::TaskView => Self::DatabasePath,
        }
    }
}

impl App {
    pub(super) fn open_config(&mut self) -> Result<()> {
        let selected_id = self.selected_todo().map(|todo| todo.id);
        if let Some(config) = &mut self.config {
            config.reload()?;
        }
        self.reconcile_task_view(selected_id);
        self.configuring = true;
        self.config_setting = ConfigSetting::DatabasePath;
        self.vim.set_mode(VimMode::Normal);
        self.message("");
        Ok(())
    }

    fn change_task_view(&mut self) -> Result<()> {
        let selected_id = self.selected_todo().map(|todo| todo.id);
        let view = self.task_view().next();
        self.config
            .as_mut()
            .context("Configuration is unavailable")?
            .save_task_view(view)?;
        self.reconcile_task_view(selected_id);
        self.message(format!("Saved. {} view applied immediately.", view.label()));
        Ok(())
    }

    pub(super) fn apply_config(&mut self, action: VimAction) -> Result<()> {
        if self.vim.input().is_some() {
            match action {
                VimAction::Cancel => {
                    self.vim.end_input();
                    self.editor.reset(String::new());
                    self.message("");
                }
                VimAction::Submit => {
                    self.editor.prepare_submit();
                    self.config
                        .as_mut()
                        .context("Configuration is unavailable")?
                        .save_database_path(self.editor.text())?;
                    self.vim.end_input();
                    self.editor.reset(String::new());
                    self.message("Saved. Applies on next launch.");
                }
                _ => {
                    self.editor.apply(action);
                    self.vim.set_mode(self.editor.mode());
                }
            }
        } else {
            self.vim.end_input();
            match action {
                VimAction::Cancel => {
                    self.configuring = false;
                    self.message("");
                }
                VimAction::SwitchPane => {
                    self.config_setting = self.config_setting.other();
                }
                VimAction::Move(Motion::Up, _) => self.config_setting = ConfigSetting::DatabasePath,
                VimAction::Move(Motion::Down, _) => self.config_setting = ConfigSetting::TaskView,
                VimAction::FileStart => self.config_setting = ConfigSetting::DatabasePath,
                VimAction::FileEnd => self.config_setting = ConfigSetting::TaskView,
                VimAction::Move(Motion::Left | Motion::Right, count)
                    if self.config_setting == ConfigSetting::TaskView =>
                {
                    if count % 2 == 1 {
                        self.change_task_view()?;
                    }
                }
                VimAction::Edit | VimAction::AddChild | VimAction::Toggle
                    if self.config_setting == ConfigSetting::TaskView =>
                {
                    self.change_task_view()?;
                }
                VimAction::Edit | VimAction::AddChild | VimAction::Toggle => {
                    let config = self
                        .config
                        .as_ref()
                        .context("Configuration is unavailable")?;
                    self.editor.reset(config.database_path.clone());
                    self.vim.begin_input(InputTarget::DatabasePath);
                    self.message("");
                }
                _ => {}
            }
        }
        Ok(())
    }
}
