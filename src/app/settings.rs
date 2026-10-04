use anyhow::{Context, Result};

use super::App;
use crate::vim_motion::{InputTarget, VimAction};

impl App {
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
