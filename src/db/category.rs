/// Category database queries.
use anyhow::Result;
use rusqlite::Connection;

use crate::types::{CategoryId, TickrCategory};

pub fn create_category(name: String, color: String, conn: &Connection) -> Result<CategoryId> {
    conn.execute(
        "INSERT INTO categories (name, color) VALUES (?1, ?2)",
        (name, color),
    )?;
    let category_id = conn.last_insert_rowid() as CategoryId;
    Ok(category_id)
}

pub fn query_category_id(name: &str, conn: &Connection) -> Result<Option<u32>> {
    let mut stmt = conn.prepare("SELECT id FROM categories WHERE name = ?1")?;
    let mut rows = stmt.query([name])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn query_category_by_id(id: CategoryId, conn: &Connection) -> Result<Option<TickrCategory>> {
    let mut stmt = conn.prepare("SELECT id, name, color FROM categories WHERE id = ?1")?;
    let mut rows = stmt.query([id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(TickrCategory {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn query_categories(conn: &Connection) -> Result<Vec<TickrCategory>> {
    let mut stmt = conn.prepare("SELECT id, name, color FROM categories")?;
    let rows = stmt.query_map([], |row| {
        Ok(TickrCategory {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
        })
    })?;
    let mut categories = Vec::new();
    for row in rows {
        categories.push(row?);
    }
    Ok(categories)
}

pub fn check_category_exists(name: String, conn: &Connection) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT 1 FROM categories WHERE name = ?1")?;
    let mut rows = stmt.query([name])?;
    Ok(rows.next()?.is_some())
}
