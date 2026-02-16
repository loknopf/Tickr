/// Database module with project, tickr, category queries and migrations.
mod category;
mod intervals;
mod migrations;
mod project;
mod tickr;

use anyhow::Result;
use rusqlite::Connection;

// Re-export all public functions
pub use category::{create_category, query_categories, query_category_by_id, query_category_id};
pub use intervals::create_interval;
pub use project::{
    check_project_exists, create_project, delete_project, query_project, query_project_by_id,
    query_project_worked_on_today, query_project_worked_on_week, query_projects,
};
pub use tickr::{
    create_tickr, delete_tickr, end_running_tickr, end_tickr, query_tickr, query_tickr_by_id,
    start_tickr, update_tickr_details,
};

/// Opens (or creates) the SQLite database and runs migrations.
pub fn init(db_path: &str) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    migrations::run_migrations(&conn)?;
    Ok(conn)
}

/// Returns the default database path inside the user's data directory.
/// Falls back to `./tickr.db` when no data dir is found.
pub fn default_db_path() -> String {
    if let Some(data_dir) = dirs::data_local_dir() {
        let tickr_dir = data_dir.join("tickr");
        std::fs::create_dir_all(&tickr_dir).ok();
        tickr_dir.join("tickr.db").to_string_lossy().into_owned()
    } else {
        "tickr.db".to_string()
    }
}
