use rusqlite::params;
use tauri::State;
use crate::models::CategoryDto;
use crate::AppState;

#[tauri::command]
pub async fn get_all_categories(state: State<'_, AppState>) -> Result<Vec<CategoryDto>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.name, c.description, c.color, c.created_at,
                    (SELECT COUNT(*) FROM patch_categories WHERE category_id = c.id) as patch_count,
                    (SELECT COUNT(*) FROM sequence_categories WHERE category_id = c.id) as sequence_count
             FROM categories c
             ORDER BY c.name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let categories = stmt
        .query_map([], |row| {
            Ok(CategoryDto {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
                created_at: row.get(4)?,
                patch_count: row.get(5)?,
                sequence_count: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(categories)
}

#[tauri::command]
pub async fn create_category(
    state: State<'_, AppState>,
    name: String,
    description: Option<String>,
    color: Option<String>,
) -> Result<CategoryDto, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    conn.execute(
        "INSERT INTO categories (name, description, color) VALUES (?1, ?2, ?3)",
        params![name, description, color],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    conn.query_row(
        "SELECT id, name, description, color, created_at FROM categories WHERE id = ?1",
        params![id],
        |row| {
            Ok(CategoryDto {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
                created_at: row.get(4)?,
                patch_count: 0,
                sequence_count: 0,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_category(
    state: State<'_, AppState>,
    id: i64,
    name: Option<String>,
    description: Option<String>,
    color: Option<String>,
) -> Result<CategoryDto, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    if let Some(n) = name {
        conn.execute(
            "UPDATE categories SET name = ?1 WHERE id = ?2",
            params![n, id],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(d) = description {
        conn.execute(
            "UPDATE categories SET description = ?1 WHERE id = ?2",
            params![d, id],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(c) = color {
        conn.execute(
            "UPDATE categories SET color = ?1 WHERE id = ?2",
            params![c, id],
        )
        .map_err(|e| e.to_string())?;
    }

    conn.query_row(
        "SELECT c.id, c.name, c.description, c.color, c.created_at,
                (SELECT COUNT(*) FROM patch_categories WHERE category_id = c.id) as patch_count,
                (SELECT COUNT(*) FROM sequence_categories WHERE category_id = c.id) as sequence_count
         FROM categories c WHERE c.id = ?1",
        params![id],
        |row| {
            Ok(CategoryDto {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
                created_at: row.get(4)?,
                patch_count: row.get(5)?,
                sequence_count: row.get(6)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_category(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    conn.execute("DELETE FROM categories WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn assign_patch_to_category(
    state: State<'_, AppState>,
    patch_id: i64,
    category_id: i64,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    conn.execute(
        "INSERT OR IGNORE INTO patch_categories (patch_id, category_id) VALUES (?1, ?2)",
        params![patch_id, category_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn remove_patch_from_category(
    state: State<'_, AppState>,
    patch_id: i64,
    category_id: i64,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    conn.execute(
        "DELETE FROM patch_categories WHERE patch_id = ?1 AND category_id = ?2",
        params![patch_id, category_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn assign_sequence_to_category(
    state: State<'_, AppState>,
    sequence_id: i64,
    category_id: i64,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    conn.execute(
        "INSERT OR IGNORE INTO sequence_categories (sequence_id, category_id) VALUES (?1, ?2)",
        params![sequence_id, category_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn remove_sequence_from_category(
    state: State<'_, AppState>,
    sequence_id: i64,
    category_id: i64,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    conn.execute(
        "DELETE FROM sequence_categories WHERE sequence_id = ?1 AND category_id = ?2",
        params![sequence_id, category_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
