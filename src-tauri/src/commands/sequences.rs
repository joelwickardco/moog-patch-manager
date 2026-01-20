use rusqlite::params;
use tauri::State;
use crate::models::{SequenceDto, SequenceFilter};
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
        "SELECT s.id, s.name, s.file_hash, s.file_size, s.notes, s.source_library,
                s.created_at, s.updated_at,
                (SELECT COUNT(*) FROM bank_sequence_slots WHERE sequence_id = s.id) as usage_count
         FROM sequences s
         WHERE 1=1"
    );
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref source_library) = filter.source_library {
        sql.push_str(" AND s.source_library = ?");
        params_vec.push(Box::new(source_library.clone()));
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
                name: row.get(1)?,
                file_hash: row.get(2)?,
                file_size: row.get(3)?,
                notes: row.get(4)?,
                source_library: row.get(5)?,
                usage_count: row.get(8)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let sequences: Vec<SequenceDto> = sequence_rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(sequences)
}

#[tauri::command]
pub async fn get_sequence_by_id(state: State<'_, AppState>, id: i64) -> Result<SequenceDto, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let seq: SequenceDto = conn
        .query_row(
            "SELECT s.id, s.name, s.file_hash, s.file_size, s.notes, s.source_library,
                    s.created_at, s.updated_at,
                    (SELECT COUNT(*) FROM bank_sequence_slots WHERE sequence_id = s.id) as usage_count
             FROM sequences s
             WHERE s.id = ?1",
            params![id],
            |row| {
                Ok(SequenceDto {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    file_hash: row.get(2)?,
                    file_size: row.get(3)?,
                    notes: row.get(4)?,
                    source_library: row.get(5)?,
                    usage_count: row.get(8)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

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
