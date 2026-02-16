/// Tickr (entry/task) database queries.
use anyhow::Result;
use chrono::{DateTime, Local};
use rusqlite::Connection;

use crate::{
    db::intervals::{query_intervals_by_tickr_id, query_intervals_by_time_range},
    types::{CategoryId, Tickr, TickrId, TickrQuery},
};

pub fn create_tickr(arg: Tickr, conn: &Connection) -> Result<TickrId> {
    conn.execute(
        "INSERT INTO entries (project_id, description, category_id) VALUES (?1, ?2, ?3)",
        (&arg.project_id, &arg.description, &arg.category_id),
    )?;
    Ok(conn.last_insert_rowid() as TickrId)
}

pub fn query_tickr(query: TickrQuery, conn: &Connection) -> Result<Vec<Tickr>> {
    match query {
        TickrQuery::ByProject(project) => query_tickr_by_project(project, conn),
        TickrQuery::ByProjectId(project_id) => query_tickr_by_project_id(project_id, conn),
        TickrQuery::ByTimeRange(start, end) => query_tickr_by_time_range(start, end, conn),
        TickrQuery::All => query_tickr_all(conn),
    }
}

pub fn query_tickr_all(conn: &Connection) -> Result<Vec<Tickr>> {
    let entries = conn.prepare("SELECT * FROM entries")?;
    let mut stmt = entries;
    let rows = stmt.query_map([], |row| {
        Ok(Tickr {
            id: Some(row.get(0)?),
            project_id: row.get(1)?,
            description: row.get(2)?,
            category_id: row.get(3)?,
            intervals: Vec::new(),
        })
    })?;
    let mut tickrs = Vec::new();
    for row in rows {
        tickrs.push(row?);
    }
    for tickr in &mut tickrs {
        if let Some(id) = tickr.id {
            tickr.intervals = query_intervals_by_tickr_id(id, conn)?;
        }
    }
    Ok(tickrs)
}

pub fn query_tickr_by_project(project: String, conn: &Connection) -> Result<Vec<Tickr>> {
    let projects = conn
        .prepare("SELECT id FROM projects WHERE name = ?1")?
        .query_map([project], |row| row.get(0))?
        .collect::<Result<Vec<u32>, _>>()?;
    if let Some(project_id) = projects.first() {
        let entries = conn.prepare("SELECT * FROM entries WHERE project_id = ?1");
        let mut stmt = entries?;
        let rows = stmt.query_map([*project_id], |row| {
            Ok(Tickr {
                id: Some(row.get(0)?),
                project_id: row.get(1)?,
                description: row.get(2)?,
                category_id: row.get(3)?,
                intervals: Vec::new(),
            })
        })?;
        let mut tickrs = Vec::new();
        for row in rows {
            tickrs.push(row?);
        }
        for tickr in &mut tickrs {
            if let Some(id) = tickr.id {
                tickr.intervals = query_intervals_by_tickr_id(id, conn)?;
            }
        }
        return Ok(tickrs);
    }
    Ok(Vec::new())
}

pub fn query_tickr_by_project_id(project_id: u32, conn: &Connection) -> Result<Vec<Tickr>> {
    let entries = conn.prepare("SELECT * FROM entries WHERE project_id = ?1")?;
    let mut stmt = entries;
    let rows = stmt.query_map([project_id], |row| {
        Ok(Tickr {
            id: Some(row.get(0)?),
            project_id: row.get(1)?,
            description: row.get(2)?,
            category_id: row.get(3)?,
            intervals: Vec::new(),
        })
    })?;
    let mut tickrs = Vec::new();
    for row in rows {
        tickrs.push(row?);
    }
    for tickr in &mut tickrs {
        if let Some(id) = tickr.id {
            tickr.intervals = query_intervals_by_tickr_id(id, conn)?;
        }
    }
    Ok(tickrs)
}

pub fn query_tickr_by_time_range(
    from: DateTime<Local>,
    to: DateTime<Local>,
    conn: &Connection,
) -> Result<Vec<Tickr>> {
    let mut result = Vec::new();
    let candiate_intervals = query_intervals_by_time_range(from, to, conn)?;
    for interval in &candiate_intervals {
        let tickr = query_tickr_by_id(interval.entry_id, conn)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Interval with entry_id {} has no corresponding tickr entry",
                interval.entry_id
            )
        })?;
        result.push(tickr);
    }
    Ok(result)
}

pub fn query_tickr_by_id(id: TickrId, conn: &Connection) -> Result<Option<Tickr>> {
    let mut stmt = conn.prepare("SELECT * FROM entries WHERE id = ?1")?;
    let mut rows = stmt.query([id])?;
    if let Some(row) = rows.next()? {
        let mut tickr = Tickr {
            id: Some(row.get(0)?),
            project_id: row.get(1)?,
            description: row.get(2)?,
            category_id: row.get(3)?,
            intervals: Vec::new(),
        };
        if let Some(id) = tickr.id {
            tickr.intervals = query_intervals_by_tickr_id(id, conn)?;
        }
        Ok(Some(tickr))
    } else {
        Ok(None)
    }
}

pub fn start_tickr(id: TickrId, conn: &Connection) -> Result<()> {
    let now = Local::now().to_rfc3339();
    conn.execute(
        "INSERT INTO intervals (entry_id, start_time) VALUES (?1, ?2)",
        rusqlite::params![id, now],
    )?;
    Ok(())
}

pub fn end_tickr(id: TickrId, conn: &Connection) -> Result<()> {
    let now = Local::now().to_rfc3339();
    conn.execute(
        "UPDATE intervals SET end_time = ?1 WHERE entry_id = ?2 AND end_time IS NULL",
        rusqlite::params![now, id],
    )?;
    Ok(())
}

pub fn update_tickr_details(
    id: TickrId,
    description: String,
    category_id: Option<CategoryId>,
    conn: &Connection,
) -> Result<()> {
    conn.execute(
        "UPDATE entries SET description = ?1, category_id = ?2 WHERE id = ?3",
        (description, category_id, id),
    )?;
    Ok(())
}

pub fn delete_tickr(id: TickrId, conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM entries WHERE id = ?1", [id])?;
    Ok(())
}

pub fn end_running_tickr(id: TickrId, conn: &Connection) -> Result<()> {
    end_tickr(id, conn)
}
