use rusqlite::params;
use tauri::State;
use crate::models::LibraryDto;
use crate::AppState;

#[tauri::command]
pub async fn get_all_libraries(state: State<'_, AppState>) -> Result<Vec<LibraryDto>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let mut stmt = conn
        .prepare(
            "SELECT l.id, l.name, l.description, l.source_filename, l.color,
                    l.patch_count, l.sequence_count, l.created_at, l.updated_at
             FROM libraries l
             ORDER BY l.name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let libraries = stmt
        .query_map([], |row| {
            Ok(LibraryDto {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                source_filename: row.get(3)?,
                color: row.get(4)?,
                patch_count: row.get(5)?,
                sequence_count: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(libraries)
}

#[tauri::command]
pub async fn get_library_by_id(state: State<'_, AppState>, id: i64) -> Result<LibraryDto, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    conn.query_row(
        "SELECT id, name, description, source_filename, color,
                patch_count, sequence_count, created_at, updated_at
         FROM libraries WHERE id = ?1",
        params![id],
        |row| {
            Ok(LibraryDto {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                source_filename: row.get(3)?,
                color: row.get(4)?,
                patch_count: row.get(5)?,
                sequence_count: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_library(
    state: State<'_, AppState>,
    id: i64,
    name: Option<String>,
    description: Option<String>,
    color: Option<String>,
) -> Result<LibraryDto, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    if let Some(n) = name {
        conn.execute(
            "UPDATE libraries SET name = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![n, id],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(d) = description {
        conn.execute(
            "UPDATE libraries SET description = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![d, id],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(c) = color {
        conn.execute(
            "UPDATE libraries SET color = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![c, id],
        )
        .map_err(|e| e.to_string())?;
    }

    conn.query_row(
        "SELECT id, name, description, source_filename, color,
                patch_count, sequence_count, created_at, updated_at
         FROM libraries WHERE id = ?1",
        params![id],
        |row| {
            Ok(LibraryDto {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                source_filename: row.get(3)?,
                color: row.get(4)?,
                patch_count: row.get(5)?,
                sequence_count: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_library(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    // Deleting library will cascade delete all associated patches and sequences
    conn.execute("DELETE FROM libraries WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}
