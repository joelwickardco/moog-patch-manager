use rusqlite::params;
use tauri::State;
use crate::models::{SequenceDto, SequenceFilter, CategoryDto};
use crate::AppState;

#[tauri::command]
pub async fn get_all_sequences(
    state: State<'_, AppState>,
    filter: Option<SequenceFilter>,
) -> Result<Vec<SequenceDto>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let filter = filter.unwrap_or_default();
    let mut sql = String::from(
        "SELECT s.id, s.library_id, l.name as library_name, s.name, s.file_hash, s.file_size,
                s.notes, s.created_at, s.updated_at
         FROM sequences s
         JOIN libraries l ON s.library_id = l.id
         WHERE 1=1"
    );
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(library_id) = filter.library_id {
        sql.push_str(" AND s.library_id = ?");
        params_vec.push(Box::new(library_id));
    }

    if let Some(ref name_contains) = filter.name_contains {
        sql.push_str(" AND s.name LIKE ?");
        params_vec.push(Box::new(format!("%{}%", name_contains)));
    }

    sql.push_str(" ORDER BY s.name COLLATE NOCASE");

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let sequence_rows = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(SequenceDto {
                id: row.get(0)?,
                library_id: row.get(1)?,
                library_name: row.get(2)?,
                name: row.get(3)?,
                file_hash: row.get(4)?,
                file_size: row.get(5)?,
                notes: row.get(6)?,
                categories: Vec::new(),
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut sequences: Vec<SequenceDto> = Vec::new();
    for seq_result in sequence_rows {
        let mut seq = seq_result.map_err(|e| e.to_string())?;
        seq.categories = get_sequence_categories(conn, seq.id)?;
        sequences.push(seq);
    }

    // Filter by category if specified
    if let Some(category_ids) = filter.category_ids {
        sequences.retain(|s| {
            s.categories.iter().any(|c| category_ids.contains(&c.id))
        });
    }

    Ok(sequences)
}

#[tauri::command]
pub async fn get_sequence_by_id(state: State<'_, AppState>, id: i64) -> Result<SequenceDto, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let mut seq: SequenceDto = conn
        .query_row(
            "SELECT s.id, s.library_id, l.name as library_name, s.name, s.file_hash, s.file_size,
                    s.notes, s.created_at, s.updated_at
             FROM sequences s
             JOIN libraries l ON s.library_id = l.id
             WHERE s.id = ?1",
            params![id],
            |row| {
                Ok(SequenceDto {
                    id: row.get(0)?,
                    library_id: row.get(1)?,
                    library_name: row.get(2)?,
                    name: row.get(3)?,
                    file_hash: row.get(4)?,
                    file_size: row.get(5)?,
                    notes: row.get(6)?,
                    categories: Vec::new(),
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    seq.categories = get_sequence_categories(conn, seq.id)?;
    Ok(seq)
}

#[tauri::command]
pub async fn update_sequence_notes(
    state: State<'_, AppState>,
    sequence_id: i64,
    notes: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    conn.execute(
        "UPDATE sequences SET notes = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![notes, sequence_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn delete_sequence(state: State<'_, AppState>, sequence_id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    conn.execute("DELETE FROM sequences WHERE id = ?1", params![sequence_id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn search_sequences(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SequenceDto>, String> {
    get_all_sequences(
        state,
        Some(SequenceFilter {
            name_contains: Some(query),
            ..Default::default()
        }),
    )
    .await
}

fn get_sequence_categories(
    conn: &rusqlite::Connection,
    sequence_id: i64,
) -> Result<Vec<CategoryDto>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.name, c.description, c.color, c.created_at,
                    (SELECT COUNT(*) FROM patch_categories WHERE category_id = c.id) as patch_count,
                    (SELECT COUNT(*) FROM sequence_categories WHERE category_id = c.id) as sequence_count
             FROM categories c
             JOIN sequence_categories sc ON c.id = sc.category_id
             WHERE sc.sequence_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let categories = stmt
        .query_map(params![sequence_id], |row| {
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
