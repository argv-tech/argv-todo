mod app;
mod cli;
mod config;
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
    let Some(options) = cli::options()? else {
        return Ok(());
    };
    let config = config::Config::load(&options.database_path, options.database_override)?;
    let path = &config.active_database;
    let database = db::Database::open(path)
        .with_context(|| format!("Could not open database {}", path.display()))?;
    let mut app = app::App::new(database)?;
    app.config = Some(config);
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
