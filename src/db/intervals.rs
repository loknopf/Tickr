use anyhow::Result;
use chrono::{DateTime, Local};
use rusqlite::Connection;

use crate::types::Interval;

pub fn query_intervals_by_tickr_id(tickr_id: u32, conn: &Connection) -> Result<Vec<Interval>> {
    let intervals = conn.prepare("SELECT * FROM intervals WHERE entry_id = ?1")?;
    let mut stmt = intervals;
    let rows = stmt.query_map([tickr_id], |row| {
        Ok(Interval {
            id: Some(row.get(0)?),
            entry_id: row.get(1)?,
            start_time: parse_required_datetime(row.get(2)?).expect("Expecting parsing of start datetime to succeed, all Db entries should be parsable."),
            end_time: parse_optional_datetime(row.get(3)?),
        })
    })?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn query_intervals_by_time_range(
    from: DateTime<Local>,
    to: DateTime<Local>,
    conn: &Connection,
) -> Result<Vec<Interval>, rusqlite::Error> {
    let intervals =
        conn.prepare("SELECT * FROM intervals WHERE start_time >= ?1 AND end_time <= ?2")?;
    let mut stmt = intervals;
    let rows = stmt.query_map([from.to_rfc3339(), to.to_rfc3339()], |row| {
        Ok(Interval {
            id: Some(row.get(0)?),
            entry_id: row.get(1)?,
            start_time: parse_required_datetime(row.get(2)?).expect("Expecting parsing of start datetime to succeed, all Db entries should be parsable."),
            end_time: parse_optional_datetime(row.get(3)?),
        })
    })?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn create_interval(interval: Interval, conn: &Connection) -> Result<Interval> {
    conn.execute(
        "INSERT INTO intervals (entry_id, start_time, end_time) VALUES (?1, ?2, ?3)",
        rusqlite::params![
            interval.entry_id,
            interval.start_time.to_rfc3339(),
            interval.end_time.map(|dt| dt.to_rfc3339()),
        ],
    )?;
    let id = conn.last_insert_rowid() as u32;
    Ok(Interval {
        id: Some(id),
        ..interval
    })
}

fn parse_required_datetime(value: Option<String>) -> Result<DateTime<Local>> {
    value
        .and_then(|raw| {
            DateTime::parse_from_rfc3339(&raw)
                .ok()
                .map(|dt| dt.with_timezone(&Local))
        })
        .ok_or_else(|| anyhow::anyhow!("Failed to parse datetime"))
}

fn parse_optional_datetime(value: Option<String>) -> Option<DateTime<Local>> {
    value.and_then(|raw| {
        DateTime::parse_from_rfc3339(&raw)
            .ok()
            .map(|dt| dt.with_timezone(&Local))
    })
}
