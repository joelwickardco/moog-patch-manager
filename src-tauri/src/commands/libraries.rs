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

#[tauri::command]
pub async fn create_library(state: State<'_, AppState>, name: String) -> Result<LibraryDto, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    // Check if library name already exists
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM libraries WHERE name = ?1)",
            params![name],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if exists {
        return Err(format!("A library named \"{}\" already exists", name));
    }

    // Create the library
    conn.execute(
        "INSERT INTO libraries (name) VALUES (?1)",
        params![name],
    )
    .map_err(|e| e.to_string())?;

    let library_id = conn.last_insert_rowid();

    // Create 16 empty banks for this library
    for bank_num in 1..=16 {
        let bank_name = format!("Bank {:02}", bank_num);
        conn.execute(
            "INSERT INTO banks (library_id, bank_number, name) VALUES (?1, ?2, ?3)",
            params![library_id, bank_num, bank_name],
        )
        .map_err(|e| e.to_string())?;

        let bank_id = conn.last_insert_rowid();

        // Create 16 empty patch slots
        for patch_num in 1..=16 {
            conn.execute(
                "INSERT INTO bank_patches (bank_id, patch_number, patch_id) VALUES (?1, ?2, NULL)",
                params![bank_id, patch_num],
            )
            .map_err(|e| e.to_string())?;
        }

        // Create 16 empty sequence slots
        for seq_num in 1..=16 {
            conn.execute(
                "INSERT INTO bank_sequences (bank_id, sequence_number, sequence_id) VALUES (?1, ?2, NULL)",
                params![bank_id, seq_num],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // Return the created library
    conn.query_row(
        "SELECT id, name, description, source_filename, color,
                patch_count, sequence_count, created_at, updated_at
         FROM libraries WHERE id = ?1",
        params![library_id],
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
