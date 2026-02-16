/// Database migrations and schema management.
use anyhow::Result;
use rusqlite::Connection;

/// Creates the initial schema if it doesn't exist yet.
pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS projects (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT    NOT NULL UNIQUE,
            created_at  TEXT    NOT NULL
        );

        CREATE TABLE IF NOT EXISTS entries (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id  INTEGER NOT NULL,
            description TEXT,
            category_id INTEGER,
            FOREIGN KEY (project_id) REFERENCES projects(id),
            FOREIGN KEY (category_id) REFERENCES categories(id)
        );

        CREATE TABLE IF NOT EXISTS categories (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT    NOT NULL UNIQUE,
            color       TEXT    NOT NULL
        );

        CREATE TABLE IF NOT EXISTS intervals (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            entry_id   INTEGER NOT NULL,
            start_time TEXT    NOT NULL,
            end_time   TEXT,
            FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE
        );
        ",
    )?;
    migrate_entries_nullable(conn)?;
    migrate_entries_add_category(conn)?;
    Ok(())
}

fn migrate_entries_nullable(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(entries)")?;
    let rows = stmt.query_map([], |row| {
        let name: String = row.get(1)?;
        let not_null: i32 = row.get(3)?;
        Ok((name, not_null))
    })?;
    let mut start_not_null = false;
    let mut categories_not_null = false;
    for row in rows {
        let (name, not_null) = row?;
        if name == "start_time" {
            start_not_null = not_null != 0;
        }
        if name == "category_id" {
            categories_not_null = not_null != 0;
        }
    }
    if !start_not_null && !categories_not_null {
        return Ok(());
    }

    conn.execute_batch(
        "
        BEGIN;
        ALTER TABLE entries RENAME TO entries_old;
        CREATE TABLE entries (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id  INTEGER NOT NULL,
            description TEXT,
            start_time  TEXT,
            end_time    TEXT,
            category_id INTEGER,
            FOREIGN KEY (project_id) REFERENCES projects(id),
            FOREIGN KEY (category_id) REFERENCES categories(id)
        );
        INSERT INTO entries (id, project_id, description, start_time, end_time, category_id)
        SELECT id, project_id, description, start_time, end_time, category_id FROM entries_old;
        DROP TABLE entries_old;
        COMMIT;
        ",
    )?;
    Ok(())
}

fn migrate_entries_add_category(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(entries)")?;
    let rows = stmt.query_map([], |row| {
        let name: String = row.get(1)?;
        Ok(name)
    })?;
    for row in rows {
        if row? == "category_id" {
            return Ok(());
        }
    }

    conn.execute("ALTER TABLE entries ADD COLUMN category_id INTEGER", [])?;
    Ok(())
}
