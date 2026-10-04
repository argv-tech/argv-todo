use anyhow::Result;
use crossterm::{
    cursor::SetCursorStyle,
    event::{self, Event},
    execute,
};
use ratatui::DefaultTerminal;

use super::App;
use crate::vim_motion::VimMode;

impl App {
    pub(crate) fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.running {
            let shape = if self.vim.mode() == VimMode::Normal {
                SetCursorStyle::SteadyBlock
            } else {
                SetCursorStyle::SteadyBar
            };
            execute!(std::io::stdout(), shape)?;
            terminal.draw(|frame| crate::ui::draw(frame, self))?;
            match event::read()? {
                Event::Key(key) => {
                    if let Some(action) = self.vim.handle(key) {
                        self.dispatch(action);
                    }
                }
                Event::Paste(text) if self.vim.mode() != VimMode::Normal && !self.help => {
                    self.editor.insert(&text);
                    self.normalize_selection();
                }
                _ => {}
            }
        }
        Ok(())
    }
}
