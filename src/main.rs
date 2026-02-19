mod app;
mod cli;
mod color;
mod db;
mod event;
mod tui;
mod types;
mod ui;
mod updater;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let db_path = db::default_db_path();
    let conn = db::init(&db_path)?;
    let cli_opts = cli::Cli::parse();
    if let Some(command) = cli_opts.command {
        return cli::run(command, &conn);
    }

    let mut app = app::App::new(conn);
    
    // Check for updates at startup
    if let Ok(Some(new_version)) = updater::check_for_updates() {
        app.show_update_popup(new_version);
    }
    
    let mut terminal = tui::init()?;
    let mut event_handler = event::EventHandler::new();
    let result = event_handler.run(&mut app, &mut terminal);

    tui::restore()?;

    // Perform update after TUI is restored if user accepted
    if app.pending_update {
        println!("Starting update process...");
        updater::perform_update()?;
    }

    result
}
