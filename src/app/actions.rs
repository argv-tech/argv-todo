use anyhow::Result;

use super::App;
use crate::vim_motion::{Motion, VimAction, VimMode};

impl App {
    pub(super) fn dispatch(&mut self, action: VimAction) {
        if let Err(error) = self.apply(action) {
            self.status = format!("Error: {error:#}");
            self.error = true;
        }
    }

    pub(super) fn apply(&mut self, action: VimAction) -> Result<()> {
        if self.help {
            match action {
                VimAction::Cancel | VimAction::Help => self.help = false,
                VimAction::Quit => self.running = false,
                VimAction::Move(Motion::Down | Motion::Right, count) => {
                    self.help_scroll = self.help_scroll.saturating_add(count as u16).min(20);
                }
                VimAction::Move(Motion::Up | Motion::Left, count) => {
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
        if self.configuring {
            return self.apply_config(action);
        }
        if self.vim.mode() != VimMode::Normal {
            return self.apply_input(action);
        }
        self.apply_tasks(action)
    }
}
