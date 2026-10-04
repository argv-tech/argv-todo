use anyhow::Result;

use super::App;
use crate::vim_motion::VimAction;

impl App {
    pub(super) fn dispatch(&mut self, action: VimAction) {
        if let Err(error) = self.apply(action) {
            self.status = format!("Error: {error:#}");
            self.error = true;
        }
    }

    pub(super) fn apply(&mut self, action: VimAction) -> Result<()> {
        if self.help {
            self.apply_help(action);
            return Ok(());
        }
        if action == VimAction::Quit {
            self.running = false;
            return Ok(());
        }
        if self.configuring {
            return self.apply_config(action);
        }
        if self.vim.input().is_some() {
            return self.apply_input(action);
        }
        self.apply_tasks(action)
    }
}
