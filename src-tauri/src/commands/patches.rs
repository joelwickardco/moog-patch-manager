use rusqlite::params;
use tauri::State;
use crate::models::{PatchDto, PatchFilter, CategoryDto};
use crate::AppState;

#[tauri::command]
pub async fn get_all_patches(
    state: State<'_, AppState>,
    filter: Option<PatchFilter>,
) -> Result<Vec<PatchDto>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let filter = filter.unwrap_or_default();
    let mut sql = String::from(
        "SELECT p.id, p.library_id, l.name as library_name, p.name, p.file_hash, p.file_size,
                p.is_favorite, p.notes, p.created_at, p.updated_at
         FROM patches p
         JOIN libraries l ON p.library_id = l.id
         WHERE 1=1"
    );
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(library_id) = filter.library_id {
        sql.push_str(" AND p.library_id = ?");
        params_vec.push(Box::new(library_id));
    }

    if let Some(is_fav) = filter.is_favorite {
        sql.push_str(" AND p.is_favorite = ?");
        params_vec.push(Box::new(is_fav));
    }

    if let Some(ref name_contains) = filter.name_contains {
        sql.push_str(" AND p.name LIKE ?");
        params_vec.push(Box::new(format!("%{}%", name_contains)));
    }

    sql.push_str(" ORDER BY p.name COLLATE NOCASE");

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let patch_rows = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(PatchDto {
                id: row.get(0)?,
                library_id: row.get(1)?,
                library_name: row.get(2)?,
                name: row.get(3)?,
                file_hash: row.get(4)?,
                file_size: row.get(5)?,
                is_favorite: row.get(6)?,
                notes: row.get(7)?,
                categories: Vec::new(), // Will be populated below
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut patches: Vec<PatchDto> = Vec::new();
    for patch_result in patch_rows {
        let mut patch = patch_result.map_err(|e| e.to_string())?;
        patch.categories = get_patch_categories(conn, patch.id)?;
        patches.push(patch);
    }

    // Filter by category if specified
    if let Some(category_ids) = filter.category_ids {
        patches.retain(|p| {
            p.categories.iter().any(|c| category_ids.contains(&c.id))
        });
    }

    Ok(patches)
}

#[tauri::command]
pub async fn get_patch_by_id(state: State<'_, AppState>, id: i64) -> Result<PatchDto, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let mut patch: PatchDto = conn
        .query_row(
            "SELECT p.id, p.library_id, l.name as library_name, p.name, p.file_hash, p.file_size,
                    p.is_favorite, p.notes, p.created_at, p.updated_at
             FROM patches p
             JOIN libraries l ON p.library_id = l.id
             WHERE p.id = ?1",
            params![id],
            |row| {
                Ok(PatchDto {
                    id: row.get(0)?,
                    library_id: row.get(1)?,
                    library_name: row.get(2)?,
                    name: row.get(3)?,
                    file_hash: row.get(4)?,
                    file_size: row.get(5)?,
                    is_favorite: row.get(6)?,
                    notes: row.get(7)?,
                    categories: Vec::new(),
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    patch.categories = get_patch_categories(conn, patch.id)?;
    Ok(patch)
}

#[tauri::command]
pub async fn toggle_favorite(state: State<'_, AppState>, patch_id: i64) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    conn.execute(
        "UPDATE patches SET is_favorite = NOT is_favorite, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        params![patch_id],
    )
    .map_err(|e| e.to_string())?;

    let is_favorite: bool = conn
        .query_row(
            "SELECT is_favorite FROM patches WHERE id = ?1",
            params![patch_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(is_favorite)
}

#[tauri::command]
pub async fn update_patch_notes(
    state: State<'_, AppState>,
    patch_id: i64,
    notes: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    conn.execute(
        "UPDATE patches SET notes = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![notes, patch_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn delete_patch(state: State<'_, AppState>, patch_id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    conn.execute("DELETE FROM patches WHERE id = ?1", params![patch_id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn search_patches(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<PatchDto>, String> {
    get_all_patches(
        state,
        Some(PatchFilter {
            name_contains: Some(query),
            ..Default::default()
        }),
    )
    .await
}

fn get_patch_categories(
    conn: &rusqlite::Connection,
    patch_id: i64,
) -> Result<Vec<CategoryDto>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.name, c.description, c.color, c.created_at,
                    (SELECT COUNT(*) FROM patch_categories WHERE category_id = c.id) as patch_count,
                    (SELECT COUNT(*) FROM sequence_categories WHERE category_id = c.id) as sequence_count
             FROM categories c
             JOIN patch_categories pc ON c.id = pc.category_id
             WHERE pc.patch_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let categories = stmt
        .query_map(params![patch_id], |row| {
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
