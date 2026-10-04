use anyhow::{Context, Result};

use super::App;
use crate::vim_motion::{Editor, VimAction, VimMode};

impl App {
    pub(super) fn apply_config(&mut self, action: VimAction) -> Result<()> {
        if self.vim.mode() != VimMode::Normal {
            match action {
                VimAction::Cancel => {
                    self.vim.set_mode(VimMode::Normal);
                    self.editor = Editor::default();
                    self.message("");
                }
                VimAction::Submit => {
                    self.config
                        .as_mut()
                        .context("Configuration is unavailable")?
                        .save_database_path(self.editor.text())?;
                    self.vim.set_mode(VimMode::Normal);
                    self.editor = Editor::default();
                    self.message("Saved. Applies on next launch.");
                }
                _ => self.editor.apply(action),
            }
        } else {
            self.vim.set_mode(VimMode::Normal);
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
                    self.editor = Editor::new(config.database_path.clone());
                    self.vim.set_mode(VimMode::Insert);
                    self.message("");
                }
                _ => {}
            }
        }
        Ok(())
    }
}
