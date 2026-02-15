mod app;
mod cli;
mod color;
mod db;
mod event;
mod tui;
mod ui;
mod types;

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
    let mut terminal = tui::init()?;
    let result = event::run(&mut app, &mut terminal);

    tui::restore()?;

    result
}
