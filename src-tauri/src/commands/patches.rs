use rusqlite::params;
use tauri::State;
use crate::models::{PatchDto, PatchFilter, TagDto};
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
        "SELECT p.id, p.name, p.file_hash, p.file_size, p.is_favorite, p.notes,
                p.source_library, p.created_at, p.updated_at,
                (SELECT COUNT(*) FROM bank_patch_slots WHERE patch_id = p.id) as usage_count,
                GROUP_CONCAT(t.name, '|||') as tag_names
         FROM patches p
         LEFT JOIN patch_tags pt ON p.id = pt.patch_id
         LEFT JOIN tags t ON pt.tag_id = t.id
         WHERE 1=1"
    );
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref source_library) = filter.source_library {
        sql.push_str(" AND p.source_library = ?");
        params_vec.push(Box::new(source_library.clone()));
    }

    if let Some(is_fav) = filter.is_favorite {
        sql.push_str(" AND p.is_favorite = ?");
        params_vec.push(Box::new(is_fav));
    }

    if let Some(ref name_contains) = filter.name_contains {
        sql.push_str(" AND p.name LIKE ?");
        params_vec.push(Box::new(format!("%{}%", name_contains)));
    }

    // Tag filtering
    if let Some(ref tag_list) = filter.tags {
        if !tag_list.is_empty() {
            sql.push_str(" AND p.id IN (SELECT pt.patch_id FROM patch_tags pt INNER JOIN tags t ON pt.tag_id = t.id WHERE t.name IN (");
            for (i, tag) in tag_list.iter().enumerate() {
                if i > 0 {
                    sql.push_str(", ");
                }
                sql.push('?');
                params_vec.push(Box::new(tag.clone()));
            }
            sql.push(')');

            // For AND logic (all tags required)
            if filter.require_all_tags.unwrap_or(false) {
                sql.push_str(&format!(
                    " GROUP BY pt.patch_id HAVING COUNT(DISTINCT t.id) = {}",
                    tag_list.len()
                ));
            }
            sql.push(')');
        }
    }

    sql.push_str(" GROUP BY p.id ORDER BY p.name COLLATE NOCASE");

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let patch_rows = stmt
        .query_map(params_refs.as_slice(), |row| {
            let tag_names_raw: Option<String> = row.get(10)?;
            let tags = tag_names_raw
                .map(|s| s.split("|||").map(|t| t.to_string()).collect())
                .unwrap_or_default();

            Ok(PatchDto {
                id: row.get(0)?,
                name: row.get(1)?,
                file_hash: row.get(2)?,
                file_size: row.get(3)?,
                is_favorite: row.get(4)?,
                notes: row.get(5)?,
                source_library: row.get(6)?,
                usage_count: row.get(9)?,
                tags,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let patches: Vec<PatchDto> = patch_rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(patches)
}

#[tauri::command]
pub async fn get_patch_by_id(state: State<'_, AppState>, id: i64) -> Result<PatchDto, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let patch: PatchDto = conn
        .query_row(
            "SELECT p.id, p.name, p.file_hash, p.file_size, p.is_favorite, p.notes,
                    p.source_library, p.created_at, p.updated_at,
                    (SELECT COUNT(*) FROM bank_patch_slots WHERE patch_id = p.id) as usage_count,
                    GROUP_CONCAT(t.name, '|||') as tag_names
             FROM patches p
             LEFT JOIN patch_tags pt ON p.id = pt.patch_id
             LEFT JOIN tags t ON pt.tag_id = t.id
             WHERE p.id = ?1
             GROUP BY p.id",
            params![id],
            |row| {
                let tag_names_raw: Option<String> = row.get(10)?;
                let tags = tag_names_raw
                    .map(|s| s.split("|||").map(|t| t.to_string()).collect())
                    .unwrap_or_default();

                Ok(PatchDto {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    file_hash: row.get(2)?,
                    file_size: row.get(3)?,
                    is_favorite: row.get(4)?,
                    notes: row.get(5)?,
                    source_library: row.get(6)?,
                    usage_count: row.get(9)?,
                    tags,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

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

#[tauri::command]
pub async fn get_patches_for_library(
    state: State<'_, AppState>,
    library_id: i64,
) -> Result<Vec<PatchDto>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    // Get all patches assigned to any bank slot in this library
    let sql = "SELECT DISTINCT p.id, p.name, p.file_hash, p.file_size, p.is_favorite, p.notes,
                p.source_library, p.created_at, p.updated_at,
                (SELECT COUNT(*) FROM bank_patch_slots WHERE patch_id = p.id) as usage_count,
                (SELECT GROUP_CONCAT(t.name, '|||') FROM patch_tags pt
                 INNER JOIN tags t ON pt.tag_id = t.id
                 WHERE pt.patch_id = p.id) as tag_names
         FROM patches p
         INNER JOIN bank_patch_slots bps ON p.id = bps.patch_id
         INNER JOIN banks b ON bps.bank_id = b.id
         WHERE b.library_id = ?1
         ORDER BY p.name COLLATE NOCASE";

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let patch_rows = stmt
        .query_map(params![library_id], |row| {
            let tag_names_raw: Option<String> = row.get(10)?;
            let tags = tag_names_raw
                .map(|s| s.split("|||").map(|t| t.to_string()).collect())
                .unwrap_or_default();

            Ok(PatchDto {
                id: row.get(0)?,
                name: row.get(1)?,
                file_hash: row.get(2)?,
                file_size: row.get(3)?,
                is_favorite: row.get(4)?,
                notes: row.get(5)?,
                source_library: row.get(6)?,
                usage_count: row.get(9)?,
                tags,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let patches: Vec<PatchDto> = patch_rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(patches)
}

// ===== Tag Management Commands =====

#[tauri::command]
pub async fn get_all_tags(state: State<'_, AppState>) -> Result<Vec<TagDto>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let mut stmt = conn
        .prepare(
            "SELECT id, name, created_at, updated_at
             FROM tags
             ORDER BY name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let tag_rows = stmt
        .query_map([], |row| {
            Ok(TagDto {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let tags: Vec<TagDto> = tag_rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(tags)
}

#[tauri::command]
pub async fn add_tag_to_patch(
    state: State<'_, AppState>,
    patch_id: i64,
    tag_name: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let tag_name = tag_name.trim();
    if tag_name.is_empty() {
        return Err("Tag name cannot be empty".to_string());
    }

    // Get or create tag (INSERT OR IGNORE ensures idempotency)
    conn.execute(
        "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
        params![tag_name],
    )
    .map_err(|e| e.to_string())?;

    // Get tag ID
    let tag_id: i64 = conn
        .query_row(
            "SELECT id FROM tags WHERE name = ?1 COLLATE NOCASE",
            params![tag_name],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Associate tag with patch (INSERT OR IGNORE prevents duplicate associations)
    conn.execute(
        "INSERT OR IGNORE INTO patch_tags (patch_id, tag_id) VALUES (?1, ?2)",
        params![patch_id, tag_id],
    )
    .map_err(|e| e.to_string())?;

    // Update patch timestamp
    conn.execute(
        "UPDATE patches SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        params![patch_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn remove_tag_from_patch(
    state: State<'_, AppState>,
    patch_id: i64,
    tag_name: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    conn.execute(
        "DELETE FROM patch_tags
         WHERE patch_id = ?1
         AND tag_id = (SELECT id FROM tags WHERE name = ?2 COLLATE NOCASE)",
        params![patch_id, tag_name],
    )
    .map_err(|e| e.to_string())?;

    // Update patch timestamp
    conn.execute(
        "UPDATE patches SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        params![patch_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn update_patch_tags(
    state: State<'_, AppState>,
    patch_id: i64,
    tag_names: Vec<String>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    // Use a transaction for atomicity
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| e.to_string())?;

    // Remove all existing tags for this patch
    tx.execute(
        "DELETE FROM patch_tags WHERE patch_id = ?1",
        params![patch_id],
    )
    .map_err(|e| e.to_string())?;

    // Add new tags
    for tag_name in tag_names {
        let tag_name = tag_name.trim();
        if tag_name.is_empty() {
            continue;
        }

        // Create tag if doesn't exist
        tx.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
            params![tag_name],
        )
        .map_err(|e| e.to_string())?;

        // Get tag ID
        let tag_id: i64 = tx
            .query_row(
                "SELECT id FROM tags WHERE name = ?1 COLLATE NOCASE",
                params![tag_name],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        // Associate with patch
        tx.execute(
            "INSERT INTO patch_tags (patch_id, tag_id) VALUES (?1, ?2)",
            params![patch_id, tag_id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Update patch timestamp
    tx.execute(
        "UPDATE patches SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        params![patch_id],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_tag_usage_counts(
    state: State<'_, AppState>,
) -> Result<Vec<(String, i64)>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let mut stmt = conn
        .prepare(
            "SELECT t.name, COUNT(pt.patch_id) as usage_count
             FROM tags t
             LEFT JOIN patch_tags pt ON t.id = pt.tag_id
             GROUP BY t.id, t.name
             ORDER BY usage_count DESC, t.name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let results: Vec<(String, i64)> = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(results)
}

#[tauri::command]
pub async fn delete_unused_tags(state: State<'_, AppState>) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let deleted = conn
        .execute(
            "DELETE FROM tags
             WHERE id NOT IN (SELECT DISTINCT tag_id FROM patch_tags)",
            [],
        )
        .map_err(|e| e.to_string())?;

    Ok(deleted as i64)
}
