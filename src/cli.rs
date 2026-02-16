/// CLI argument parsing and command handling.
use anyhow::Result;
use chrono::{DateTime, Local};
use clap::{Parser, Subcommand};
use rusqlite::Connection;

use crate::{db, types};

#[derive(Parser)]
#[command(
    name = "tickr",
    version,
    about = "Tickr - A terminal-based time tracker"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Project {
        #[command(subcommand)]
        command: ProjectCommand,
    },
    Task {
        #[command(subcommand)]
        command: TaskCommand,
    },
    Category {
        name: String,
        color_opt: Option<String>,
    },
    Export {
        /// Output file path for CSV export
        #[arg(short = 'o', long = "output", default_value = "tickr_export.csv")]
        output: String,
        
        /// Start date for export (RFC3339 format, e.g., 2024-01-01T00:00:00+00:00)
        #[arg(short = 's', long = "start")]
        start: Option<String>,
        
        /// End date for export (RFC3339 format, e.g., 2024-12-31T23:59:59+00:00)
        #[arg(short = 'e', long = "end")]
        end: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProjectCommand {
    Add { name: String },
}

#[derive(Subcommand, Debug)]
pub enum TaskCommand {
    Add {
        project: String,
        description: String,
        #[arg(short = 's', long = "start")]
        start: Option<String>,
        #[arg(short = 'e', long = "end")]
        end: Option<String>,
        #[arg(short = 'c', long = "category")]
        category: Option<String>,
    },
    Switch {
        project: String,
        description: String,
    },
    Start {
        project: String,
        description: String,
    },
}

/// Execute a CLI command (project, task, or category).
pub fn run(command: Command, conn: &Connection) -> Result<()> {
    match command {
        Command::Project {
            command: ProjectCommand::Add { name },
        } => handle_project_add(name, conn)?,
        Command::Task {
            command:
                TaskCommand::Add {
                    project,
                    description,
                    start,
                    end,
                    category,
                },
        } => handle_task_add(project, description, start, end, category, conn)?,
        Command::Task {
            command:
                TaskCommand::Switch {
                    project,
                    description,
                },
        } => handle_task_switch(project, description, conn)?,
        Command::Task {
            command:
                TaskCommand::Start {
                    project,
                    description,
                },
        } => handle_task_switch(project, description, conn)?, // Starting a task is the same as switching to it if no other is currently running
        Command::Category { name, color_opt } => handle_category_add(name, color_opt, conn)?,
        Command::Export { output, start, end } => handle_export(output, start, end, conn)?,
    }
    Ok(())
}

fn handle_project_add(name: String, conn: &Connection) -> Result<()> {
    if db::check_project_exists(&name, conn)? {
        println!("Project '{name}' already exists.");
        return Ok(());
    }
    db::create_project(
        types::Project {
            id: None,
            name,
            created_at: Local::now(),
        },
        conn,
    )?;
    Ok(())
}

fn handle_task_add(
    project: String,
    description: String,
    start: Option<String>,
    end: Option<String>,
    category: Option<String>,
    conn: &Connection,
) -> Result<()> {
    let projects = db::query_project(types::ProjectQuery::ByName(project.clone()), conn)?;
    if projects.is_empty() {
        println!("Project '{project}' not found");
        return Ok(());
    }
    if projects.len() > 1 {
        println!("Multiple projects found with the same name, cannot determine which one to use");
        return Ok(());
    }
    let project_id = projects[0].id.unwrap();

    let start_time = parse_optional_datetime(start)?;
    let end_time = parse_optional_datetime(end)?;
    if start_time.is_none() && end_time.is_some() {
        println!("End time requires a start time.");
        return Ok(());
    }

    let category_id = if let Some(cat_name) = category {
        match db::query_category_id(&cat_name, conn)? {
            Some(id) => Some(id),
            None => {
                println!("Category '{cat_name}' not found, creating it with a random color.");
                let color = crate::color::random_color();
                Some(db::create_category(cat_name, color, conn)?)
            }
        }
    } else {
        None
    };

    let tickr_id = db::create_tickr(
        types::Tickr {
            id: None,
            project_id,
            description,
            category_id,
            intervals: Vec::new(), // Intervals will be created separately based on start/end times
        },
        conn,
    )?;
    if let Some(start_time) = start_time {
        db::create_interval(
            types::Interval {
                id: None,
                entry_id: tickr_id,
                start_time,
                end_time,
            },
            conn,
        )?;
    }
    Ok(())
}

fn handle_task_switch(project: String, description: String, conn: &Connection) -> Result<()> {
    let projects = db::query_project(types::ProjectQuery::ByName(project.clone()), conn)?;
    if projects.is_empty() {
        println!("Project '{project}' not found");
        return Ok(());
    }
    if projects.len() > 1 {
        println!("Multiple projects found with the same name, cannot determine which one to use");
        return Ok(());
    }
    let project_id = projects[0].id.unwrap();
    let tickrs = db::query_tickr(types::TickrQuery::ByProjectId(project_id), conn)?;
    let mut tickr = None;
    for tickr_candidate in tickrs {
        if tickr_candidate.description == description {
            println!("Switching to task '{}'", description);
            tickr = Some(tickr_candidate);
            break;
        }
    }
    if tickr.is_none() {
        println!("Task '{}' not found in project '{}'", description, project);
        return Ok(());
    }
    let tickr = tickr.unwrap();
    let tickr_to_stop = db::query_tickr(types::TickrQuery::ByProjectId(project_id), conn)?
        .into_iter()
        .find(|t| t.intervals.iter().any(|i| i.end_time.is_none()));
    if let Some(old_tickr) = tickr_to_stop {
        println!(
            "Stopping currently running task '{}'",
            old_tickr.description
        );
        db::end_tickr(old_tickr.id.unwrap(), conn)?;
    }
    db::start_tickr(tickr.id.unwrap(), conn)?;
    Ok(())
}

fn handle_category_add(name: String, color_opt: Option<String>, conn: &Connection) -> Result<()> {
    let color = if let Some(c) = color_opt {
        if !crate::color::is_valid_hex(&c) {
            println!("Invalid color format. Please provide a hex code like #RRGGBB.");
            return Ok(());
        }
        c
    } else {
        crate::color::random_color()
    };
    db::create_category(name, color, conn)?;
    Ok(())
}

fn parse_optional_datetime(value: Option<String>) -> Result<Option<DateTime<Local>>> {
    match value {
        Some(s) => {
            let dt = DateTime::parse_from_rfc3339(&s)?.with_timezone(&Local);
            Ok(Some(dt))
        }
        None => Ok(None),
    }
}

fn handle_export(
    output: String,
    start: Option<String>,
    end: Option<String>,
    conn: &Connection,
) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

    let start_time = parse_optional_datetime(start)?;
    let end_time = parse_optional_datetime(end)?;

    // Get all tickrs
    let tickrs = db::query_tickr(types::TickrQuery::All, conn)?;
    let projects = db::query_projects(conn)?;
    let categories = db::query_categories(conn)?;

    // Create CSV file
    let mut file = File::create(&output)?;

    // Write CSV header
    writeln!(
        file,
        "Project,Task,Category,Start Time,End Time,Duration (seconds)"
    )?;

    let mut total_exported = 0;

    // Write data rows
    for tickr in &tickrs {
        let project_name = projects
            .iter()
            .find(|p| p.id == Some(tickr.project_id))
            .map(|p| p.name.as_str())
            .unwrap_or("Unknown");

        let category_name = tickr
            .category_id
            .and_then(|cat_id| {
                categories
                    .iter()
                    .find(|c| c.id == cat_id)
                    .map(|c| c.name.as_str())
            })
            .unwrap_or("");

        for interval in &tickr.intervals {
            // Filter by date range if provided
            if let Some(start) = start_time {
                if interval.start_time < start {
                    continue;
                }
            }
            if let Some(end) = end_time {
                if interval.start_time > end {
                    continue;
                }
            }

            let start_str = interval.start_time.to_rfc3339();
            let end_str = interval
                .end_time
                .map(|e| e.to_rfc3339())
                .unwrap_or_else(|| "Running".to_string());

            let duration = if let Some(end_time) = interval.end_time {
                end_time
                    .signed_duration_since(interval.start_time)
                    .num_seconds()
            } else {
                Local::now()
                    .signed_duration_since(interval.start_time)
                    .num_seconds()
            };

            writeln!(
                file,
                "{},{},{},{},{},{}",
                escape_csv(project_name),
                escape_csv(&tickr.description),
                escape_csv(category_name),
                start_str,
                end_str,
                duration
            )?;

            total_exported += 1;
        }
    }

    println!("Exported {} intervals to {}", total_exported, output);
    Ok(())
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
