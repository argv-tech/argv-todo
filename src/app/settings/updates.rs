use anyhow::{Context, Result};

use super::ConfigSetting;
use crate::{
    app::App,
    config::TaskView,
    vim_motion::{InputTarget, Motion, VimAction, VimMode},
};

impl App {
    pub(in crate::app) fn open_config(&mut self) -> Result<()> {
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

    fn change_setting(&mut self, forward: bool, count: usize) -> Result<()> {
        if self.config_setting == ConfigSetting::ShowCompleted
            && self.task_view() == TaskView::Split
        {
            self.message("show_completed is always false in Split; use the Completed pane.");
            return Ok(());
        }
        let selected_id = self.selected_todo().map(|todo| todo.id);
        let config = self
            .config
            .as_mut()
            .context("Configuration is unavailable")?;
        let steps = if self.config_setting == ConfigSetting::DefaultPriority {
            if forward {
                count % 3
            } else {
                (3 - count % 3) % 3
            }
        } else {
            count % 2
        };
        if steps == 0 {
            return Ok(());
        }
        match self.config_setting {
            ConfigSetting::DatabasePath => return Ok(()),
            ConfigSetting::TaskView => config.save_task_view(config.task_view.next())?,
            ConfigSetting::DefaultPriority => {
                let mut priority = config.default_priority;
                for _ in 0..steps {
                    priority = priority.next();
                }
                config.save_default_priority(priority)?;
            }
            ConfigSetting::ShowCompleted => config.save_show_completed(!config.show_completed)?,
            ConfigSetting::SortOrder => config.save_sort_order(config.sort_order.next())?,
            ConfigSetting::ShowHints => config.save_show_hints(!config.show_hints)?,
        }
        self.reconcile_task_view(selected_id);
        self.message(format!(
            "Saved. {} = {}.",
            self.config_setting.name(),
            self.config_setting.value(self)
        ));
        Ok(())
    }

    pub(in crate::app) fn apply_config(&mut self, action: VimAction) -> Result<()> {
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
            return Ok(());
        }
        match action {
            VimAction::Cancel => {
                self.configuring = false;
                self.message("");
            }
            VimAction::SwitchPane | VimAction::SwitchPaneBackward => {
                self.config_setting =
                    self.config_setting
                        .step(action == VimAction::SwitchPane, 1, true);
            }
            VimAction::Move(Motion::Up | Motion::Down, count) => {
                self.config_setting = self.config_setting.step(
                    matches!(action, VimAction::Move(Motion::Down, _)),
                    count,
                    false,
                );
            }
            VimAction::FileStart => self.config_setting = ConfigSetting::DatabasePath,
            VimAction::FileEnd => self.config_setting = ConfigSetting::ShowHints,
            VimAction::Move(Motion::Left | Motion::Right, count)
                if self.config_setting != ConfigSetting::DatabasePath =>
            {
                self.change_setting(matches!(action, VimAction::Move(Motion::Right, _)), count)?;
            }
            VimAction::Edit | VimAction::AddChild | VimAction::Toggle => {
                if self.config_setting == ConfigSetting::DatabasePath {
                    let config = self
                        .config
                        .as_ref()
                        .context("Configuration is unavailable")?;
                    self.editor.reset(config.database_path.clone());
                    self.vim.begin_input(InputTarget::DatabasePath);
                    self.message("");
                } else {
                    self.change_setting(true, 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
