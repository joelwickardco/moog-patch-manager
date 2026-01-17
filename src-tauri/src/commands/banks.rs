use rusqlite::params;
use tauri::State;
use crate::models::{BankDto, PatchDto, SequenceDto};
use crate::AppState;

#[tauri::command]
pub async fn get_all_banks(state: State<'_, AppState>) -> Result<Vec<BankDto>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let mut banks = Vec::new();

    let mut stmt = conn
        .prepare(
            "SELECT id, bank_number, name, description, created_at, updated_at
             FROM banks ORDER BY bank_number",
        )
        .map_err(|e| e.to_string())?;

    let bank_rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    for bank_result in bank_rows {
        let (id, bank_number, name, description, created_at, updated_at) =
            bank_result.map_err(|e| e.to_string())?;

        let patches = get_bank_patches(conn, id)?;
        let sequences = get_bank_sequences(conn, id)?;

        banks.push(BankDto {
            id,
            bank_number,
            name,
            description,
            patches,
            sequences,
            created_at,
            updated_at,
        });
    }

    Ok(banks)
}

#[tauri::command]
pub async fn get_bank_by_number(
    state: State<'_, AppState>,
    bank_number: i32,
) -> Result<BankDto, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let (id, name, description, created_at, updated_at): (i64, String, Option<String>, String, String) =
        conn.query_row(
            "SELECT id, name, description, created_at, updated_at FROM banks WHERE bank_number = ?1",
            params![bank_number],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .map_err(|e| e.to_string())?;

    let patches = get_bank_patches(conn, id)?;
    let sequences = get_bank_sequences(conn, id)?;

    Ok(BankDto {
        id,
        bank_number,
        name,
        description,
        patches,
        sequences,
        created_at,
        updated_at,
    })
}

#[tauri::command]
pub async fn update_bank_name(
    state: State<'_, AppState>,
    bank_number: i32,
    name: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    conn.execute(
        "UPDATE banks SET name = ?1, updated_at = CURRENT_TIMESTAMP WHERE bank_number = ?2",
        params![name, bank_number],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn assign_patch_to_bank(
    state: State<'_, AppState>,
    bank_number: i32,
    patch_number: i32,
    patch_id: Option<i64>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let bank_id: i64 = conn
        .query_row(
            "SELECT id FROM banks WHERE bank_number = ?1",
            params![bank_number],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE bank_patches SET patch_id = ?1 WHERE bank_id = ?2 AND patch_number = ?3",
        params![patch_id, bank_id, patch_number],
    )
    .map_err(|e| e.to_string())?;

    // Update bank timestamp
    conn.execute(
        "UPDATE banks SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        params![bank_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn assign_sequence_to_bank(
    state: State<'_, AppState>,
    bank_number: i32,
    sequence_number: i32,
    sequence_id: Option<i64>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let bank_id: i64 = conn
        .query_row(
            "SELECT id FROM banks WHERE bank_number = ?1",
            params![bank_number],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE bank_sequences SET sequence_id = ?1 WHERE bank_id = ?2 AND sequence_number = ?3",
        params![sequence_id, bank_id, sequence_number],
    )
    .map_err(|e| e.to_string())?;

    // Update bank timestamp
    conn.execute(
        "UPDATE banks SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        params![bank_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn clear_bank_slot(
    state: State<'_, AppState>,
    bank_number: i32,
    patch_number: i32,
) -> Result<(), String> {
    assign_patch_to_bank(state, bank_number, patch_number, None).await
}

fn get_bank_patches(
    conn: &rusqlite::Connection,
    bank_id: i64,
) -> Result<Vec<Option<PatchDto>>, String> {
    let mut patches = vec![None; 16];

    let mut stmt = conn
        .prepare(
            "SELECT bp.patch_number, p.id, p.library_id, l.name as library_name,
                    p.name, p.file_hash, p.file_size, p.is_favorite,
                    p.notes, p.created_at, p.updated_at
             FROM bank_patches bp
             LEFT JOIN patches p ON bp.patch_id = p.id
             LEFT JOIN libraries l ON p.library_id = l.id
             WHERE bp.bank_id = ?1
             ORDER BY bp.patch_number",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![bank_id], |row| {
            let patch_number: i32 = row.get(0)?;
            let patch_id: Option<i64> = row.get(1)?;

            if patch_id.is_some() {
                Ok(Some((
                    patch_number,
                    PatchDto {
                        id: row.get(1)?,
                        library_id: row.get(2)?,
                        library_name: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                        name: row.get(4)?,
                        file_hash: row.get(5)?,
                        file_size: row.get(6)?,
                        is_favorite: row.get(7)?,
                        notes: row.get(8)?,
                        categories: Vec::new(),
                        created_at: row.get(9)?,
                        updated_at: row.get(10)?,
                    },
                )))
            } else {
                Ok(None)
            }
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        if let Some((patch_number, patch)) = row.map_err(|e| e.to_string())? {
            let idx = (patch_number - 1) as usize;
            if idx < 16 {
                patches[idx] = Some(patch);
            }
        }
    }

    Ok(patches)
}

fn get_bank_sequences(
    conn: &rusqlite::Connection,
    bank_id: i64,
) -> Result<Vec<Option<SequenceDto>>, String> {
    let mut sequences = vec![None; 16];

    let mut stmt = conn
        .prepare(
            "SELECT bs.sequence_number, s.id, s.library_id, l.name as library_name,
                    s.name, s.file_hash, s.file_size, s.notes, s.created_at, s.updated_at
             FROM bank_sequences bs
             LEFT JOIN sequences s ON bs.sequence_id = s.id
             LEFT JOIN libraries l ON s.library_id = l.id
             WHERE bs.bank_id = ?1
             ORDER BY bs.sequence_number",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![bank_id], |row| {
            let seq_number: i32 = row.get(0)?;
            let seq_id: Option<i64> = row.get(1)?;

            if seq_id.is_some() {
                Ok(Some((
                    seq_number,
                    SequenceDto {
                        id: row.get(1)?,
                        library_id: row.get(2)?,
                        library_name: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                        name: row.get(4)?,
                        file_hash: row.get(5)?,
                        file_size: row.get(6)?,
                        notes: row.get(7)?,
                        categories: Vec::new(),
                        created_at: row.get(8)?,
                        updated_at: row.get(9)?,
                    },
                )))
            } else {
                Ok(None)
            }
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        if let Some((seq_number, seq)) = row.map_err(|e| e.to_string())? {
            let idx = (seq_number - 1) as usize;
            if idx < 16 {
                sequences[idx] = Some(seq);
            }
        }
    }

    Ok(sequences)
}
