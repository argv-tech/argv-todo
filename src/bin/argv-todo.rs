use anyhow::{Context, Result};
use argv_todo::{app::App, cli, config::Config, db::Database};
#[cfg(not(windows))]
use crossterm::event::{DisableBracketedPaste, EnableBracketedPaste};
use crossterm::{cursor::SetCursorStyle, execute};

fn main() -> Result<()> {
    let Some(options) = cli::options()? else {
        return Ok(());
    };
    let config = Config::load(&options.database_path, options.database_override)?;
    let path = &config.active_database;
    let database = Database::open(path)
        .with_context(|| format!("Could not open database {}", path.display()))?;
    let mut app = App::new(database)?;
    app.config = Some(config);
    app.push_notify_schedule();
    ratatui::run(|terminal| -> Result<()> {
        let _guard = TerminalInputGuard;
        // Crossterm 0.29 reads native Windows key events, not bracketed-paste sequences.
        #[cfg(not(windows))]
        execute!(std::io::stdout(), EnableBracketedPaste)?;
        app.run(terminal)
    })
}

struct TerminalInputGuard;
impl Drop for TerminalInputGuard {
    fn drop(&mut self) {
        #[cfg(not(windows))]
        let _ = execute!(std::io::stdout(), DisableBracketedPaste);
        let _ = execute!(std::io::stdout(), SetCursorStyle::DefaultUserShape);
    }
}
