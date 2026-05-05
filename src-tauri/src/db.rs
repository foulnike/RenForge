use std::collections::HashMap;
use std::path::Path;
use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::{DbEntry, FileStats};

pub fn get_db_conn(project_path: &str) -> Result<Connection, AppError> {
    let db_path = Path::new(project_path).join("renforge.db");
    let conn = Connection::open(db_path)?;
    
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;"
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS translations (
            id TEXT PRIMARY KEY,
            file_path TEXT,
            original TEXT,
            translation TEXT,
            status TEXT
        )",
        [],
    )?;
    
    Ok(conn)
}

#[tauri::command]
pub fn search_in_db(project_path: String, query: String) -> Result<Vec<DbEntry>, AppError> {
    let conn = get_db_conn(&project_path)?;
    let mut stmt = conn.prepare(
        "SELECT id, file_path, original, translation, status FROM translations 
         WHERE original LIKE ?1 OR translation LIKE ?1 LIMIT 100"
    )?;

    let rows = stmt.query_map(params![format!("%{}%", query)], |row| {
        Ok(DbEntry {
            id: row.get(0)?,
            file_path: row.get(1)?,
            original: row.get(2)?,
            translation: row.get(3)?,
            status: row.get(4)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

#[tauri::command]
pub fn get_translation_stats(project_path: String) -> Result<HashMap<String, FileStats>, AppError> {
    let conn = get_db_conn(&project_path)?;
    let mut stmt = conn.prepare(
        "SELECT file_path, COUNT(id), SUM(CASE WHEN status = 'translated' THEN 1 ELSE 0 END) 
         FROM translations GROUP BY file_path"
    )?;

    let rows = stmt.query_map([], |row| {
        let path: String = row.get(0)?;
        let total: i32 = row.get(1)?;
        let translated: i32 = row.get(2).unwrap_or(0);
        Ok((path, FileStats { total, translated }))
    })?;

    let mut map = HashMap::new();
    for row in rows {
        let (path, stats) = row?;
        map.insert(path, stats);
    }
    Ok(map)
}

#[tauri::command]
pub fn upsert_translation(project_path: String, entry: DbEntry) -> Result<(), AppError> {
    let conn = get_db_conn(&project_path)?;
    conn.execute(
        "INSERT OR REPLACE INTO translations (id, file_path, original, translation, status)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![entry.id, entry.file_path, entry.original, entry.translation, entry.status],
    )?;
    Ok(())
}

#[tauri::command]
pub fn upsert_translations_batch(project_path: String, entries: Vec<DbEntry>) -> Result<(), AppError> {
    let mut conn = get_db_conn(&project_path)?;
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO translations (id, file_path, original, translation, status)
             VALUES (?1, ?2, ?3, ?4, ?5)"
        )?;

        for entry in entries {
            stmt.execute(params![
                entry.id, entry.file_path, entry.original, entry.translation, entry.status
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}