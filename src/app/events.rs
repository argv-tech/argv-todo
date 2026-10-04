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
            let shape = match self.vim.mode() {
                VimMode::Insert => SetCursorStyle::SteadyBar,
                VimMode::Replace => SetCursorStyle::SteadyUnderScore,
                _ => SetCursorStyle::SteadyBlock,
            };
            execute!(std::io::stdout(), shape)?;
            terminal.draw(|frame| crate::ui::draw(frame, self))?;
            match event::read()? {
                Event::Key(key) => {
                    if let Some(action) = self.vim.handle(key) {
                        self.dispatch(action);
                    }
                }
                Event::Paste(text) if self.vim.input().is_some() && !self.help => {
                    self.editor.insert(&text);
                    self.vim.set_mode(self.editor.mode());
                    self.normalize_selection();
                }
                _ => {}
            }
        }
        Ok(())
    }
}
