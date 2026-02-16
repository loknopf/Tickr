/// Project-related database queries.
use anyhow::Result;
use chrono::DateTime;
use chrono::Local;
use rusqlite::Connection;

use crate::types::{Project, ProjectQuery};

pub fn create_project(arg: Project, conn: &Connection) -> Result<()> {
    conn.execute(
        "INSERT INTO projects (name, created_at) VALUES (?1, ?2)",
        (&arg.name, arg.created_at.to_rfc3339()),
    )?;
    Ok(())
}

pub fn query_projects(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare("SELECT * FROM projects")?;
    let rows = stmt.query_map([], |row| {
        Ok(Project {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                .unwrap()
                .with_timezone(&Local),
        })
    })?;
    let mut projects = Vec::new();
    for row in rows {
        projects.push(row?);
    }
    Ok(projects)
}

pub fn query_project(query: ProjectQuery, conn: &Connection) -> Result<Vec<Project>> {
    match query {
        ProjectQuery::ByName(name) => {
            query_project_by_name(name, conn).map(|opt| opt.into_iter().collect())
        }
        _ => query_projects(conn),
    }
}

pub fn query_project_by_name(name: String, conn: &Connection) -> Result<Option<Project>> {
    let mut stmt = conn.prepare("SELECT * FROM projects WHERE name = ?1")?;
    let mut rows = stmt.query([name])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Project {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                .unwrap()
                .with_timezone(&Local),
        }))
    } else {
        Ok(None)
    }
}

pub fn query_project_by_id(id: u32, conn: &Connection) -> Result<Option<Project>> {
    let mut stmt = conn.prepare("SELECT * FROM projects WHERE id = ?1")?;
    let mut rows = stmt.query([id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Project {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                .unwrap()
                .with_timezone(&Local),
        }))
    } else {
        Ok(None)
    }
}

pub fn query_project_worked_on_today(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "
        SELECT DISTINCT p.id, p.name, p.created_at
        FROM projects p
        JOIN entries e ON e.project_id = p.id
        JOIN intervals i ON i.entry_id = e.id
        WHERE i.start_time >= date('now', 'localtime') || 'T00:00:00'
        AND i.start_time <  date('now', 'localtime', '+1 day') || 'T00:00:00';",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Project {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                .unwrap()
                .with_timezone(&Local),
        })
    })?;
    let mut projects = Vec::new();
    for row in rows {
        projects.push(row?);
    }
    Ok(projects)
}

pub fn query_project_worked_on_week(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "
        SELECT DISTINCT p.id, p.name, p.created_at
        FROM projects p
        JOIN entries e ON e.project_id = p.id
        JOIN intervals i ON i.entry_id = e.id
        WHERE i.start_time >= date('now', 'localtime', '-6 day') || 'T00:00:00'
        AND i.start_time <  date('now', 'localtime', '+1 day') || 'T00:00:00';",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Project {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                .unwrap()
                .with_timezone(&Local),
        })
    })?;
    let mut projects = Vec::new();
    for row in rows {
        projects.push(row?);
    }
    Ok(projects)
}

pub fn check_project_exists(name: &str, conn: &Connection) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM projects WHERE name = ?1")?;
    let count: i64 = stmt.query_row([name], |row| row.get(0))?;
    Ok(count > 0)
}

pub fn delete_project(id: u32, conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM projects WHERE id = ?1", [id])?;
    Ok(())
}
