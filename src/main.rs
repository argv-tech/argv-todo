mod app;
mod cli;
mod db;
mod ui;
mod vim_motion;

use anyhow::{Context, Result};
use crossterm::{
    cursor::SetCursorStyle,
    event::{DisableBracketedPaste, EnableBracketedPaste},
    execute,
};

fn main() -> Result<()> {
    let Some(path) = cli::database_path()? else {
        return Ok(());
    };
    let database = db::Database::open(&path)
        .with_context(|| format!("Could not open database {}", path.display()))?;
    let mut app = app::App::new(database)?;
    ratatui::run(|terminal| -> Result<()> {
        let _guard = TerminalInputGuard;
        execute!(std::io::stdout(), EnableBracketedPaste)?;
        app.run(terminal)
    })
}

struct TerminalInputGuard;
impl Drop for TerminalInputGuard {
    fn drop(&mut self) {
        let _ = execute!(
            std::io::stdout(),
            DisableBracketedPaste,
            SetCursorStyle::DefaultUserShape
        );
    }
}
