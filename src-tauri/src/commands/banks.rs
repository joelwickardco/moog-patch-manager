use rusqlite::params;
use tauri::State;
use crate::models::{BankDto, BankSlotDto, PatchDto, SequenceDto};
use crate::AppState;

#[tauri::command]
pub async fn get_banks_for_library(
    state: State<'_, AppState>,
    library_id: i64,
) -> Result<Vec<BankDto>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let mut banks = Vec::new();

    let mut stmt = conn
        .prepare(
            "SELECT id, library_id, bank_number, name, description, created_at, updated_at
             FROM banks WHERE library_id = ?1 ORDER BY bank_number",
        )
        .map_err(|e| e.to_string())?;

    let bank_rows = stmt
        .query_map(params![library_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    for bank_result in bank_rows {
        let (id, lib_id, bank_number, name, description, created_at, updated_at) =
            bank_result.map_err(|e| e.to_string())?;

        let patch_slots = get_bank_patch_slots(conn, id)?;
        let sequence_slots = get_bank_sequence_slots(conn, id)?;

        banks.push(BankDto {
            id,
            library_id: lib_id,
            bank_number,
            name,
            description,
            patch_slots,
            sequence_slots,
            created_at,
            updated_at,
        });
    }

    Ok(banks)
}

#[tauri::command]
pub async fn get_bank_by_number(
    state: State<'_, AppState>,
    library_id: i64,
    bank_number: i32,
) -> Result<BankDto, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let (id, name, description, created_at, updated_at): (i64, String, Option<String>, String, String) =
        conn.query_row(
            "SELECT id, name, description, created_at, updated_at
             FROM banks WHERE library_id = ?1 AND bank_number = ?2",
            params![library_id, bank_number],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .map_err(|e| e.to_string())?;

    let patch_slots = get_bank_patch_slots(conn, id)?;
    let sequence_slots = get_bank_sequence_slots(conn, id)?;

    Ok(BankDto {
        id,
        library_id,
        bank_number,
        name,
        description,
        patch_slots,
        sequence_slots,
        created_at,
        updated_at,
    })
}

#[tauri::command]
pub async fn update_bank_name(
    state: State<'_, AppState>,
    library_id: i64,
    bank_number: i32,
    name: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    conn.execute(
        "UPDATE banks SET name = ?1, updated_at = CURRENT_TIMESTAMP
         WHERE library_id = ?2 AND bank_number = ?3",
        params![name, library_id, bank_number],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn assign_patch_to_slot(
    state: State<'_, AppState>,
    library_id: i64,
    bank_number: i32,
    slot_number: i32,
    patch_id: Option<i64>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let bank_id: i64 = conn
        .query_row(
            "SELECT id FROM banks WHERE library_id = ?1 AND bank_number = ?2",
            params![library_id, bank_number],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Use INSERT OR REPLACE since we have composite primary key
    conn.execute(
        "INSERT OR REPLACE INTO bank_patch_slots (bank_id, slot_number, patch_id) VALUES (?1, ?2, ?3)",
        params![bank_id, slot_number, patch_id],
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
pub async fn assign_sequence_to_slot(
    state: State<'_, AppState>,
    library_id: i64,
    bank_number: i32,
    slot_number: i32,
    sequence_id: Option<i64>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    let bank_id: i64 = conn
        .query_row(
            "SELECT id FROM banks WHERE library_id = ?1 AND bank_number = ?2",
            params![library_id, bank_number],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Use INSERT OR REPLACE since we have composite primary key
    conn.execute(
        "INSERT OR REPLACE INTO bank_sequence_slots (bank_id, slot_number, sequence_id) VALUES (?1, ?2, ?3)",
        params![bank_id, slot_number, sequence_id],
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
pub async fn clear_patch_slot(
    state: State<'_, AppState>,
    library_id: i64,
    bank_number: i32,
    slot_number: i32,
) -> Result<(), String> {
    assign_patch_to_slot(state, library_id, bank_number, slot_number, None).await
}

#[tauri::command]
pub async fn clear_sequence_slot(
    state: State<'_, AppState>,
    library_id: i64,
    bank_number: i32,
    slot_number: i32,
) -> Result<(), String> {
    assign_sequence_to_slot(state, library_id, bank_number, slot_number, None).await
}

fn get_bank_patch_slots(
    conn: &rusqlite::Connection,
    bank_id: i64,
) -> Result<Vec<BankSlotDto<PatchDto>>, String> {
    let mut slots: Vec<BankSlotDto<PatchDto>> = (1..=16)
        .map(|n| BankSlotDto {
            slot_number: n,
            content: None,
        })
        .collect();

    let mut stmt = conn
        .prepare(
            "SELECT bps.slot_number, p.id, p.name, p.file_hash, p.file_size, p.is_favorite,
                    p.notes, p.source_library, p.created_at, p.updated_at,
                    (SELECT COUNT(*) FROM bank_patch_slots WHERE patch_id = p.id) as usage_count
             FROM bank_patch_slots bps
             LEFT JOIN patches p ON bps.patch_id = p.id
             WHERE bps.bank_id = ?1
             ORDER BY bps.slot_number",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![bank_id], |row| {
            let slot_number: i32 = row.get(0)?;
            let patch_id: Option<i64> = row.get(1)?;

            if patch_id.is_some() {
                Ok(Some((
                    slot_number,
                    PatchDto {
                        id: row.get(1)?,
                        name: row.get(2)?,
                        file_hash: row.get(3)?,
                        file_size: row.get(4)?,
                        is_favorite: row.get(5)?,
                        notes: row.get(6)?,
                        source_library: row.get(7)?,
                        categories: Vec::new(),
                        usage_count: row.get(10)?,
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
        if let Some((slot_number, patch)) = row.map_err(|e| e.to_string())? {
            let idx = (slot_number - 1) as usize;
            if idx < 16 {
                slots[idx].content = Some(patch);
            }
        }
    }

    Ok(slots)
}

fn get_bank_sequence_slots(
    conn: &rusqlite::Connection,
    bank_id: i64,
) -> Result<Vec<BankSlotDto<SequenceDto>>, String> {
    let mut slots: Vec<BankSlotDto<SequenceDto>> = (1..=16)
        .map(|n| BankSlotDto {
            slot_number: n,
            content: None,
        })
        .collect();

    let mut stmt = conn
        .prepare(
            "SELECT bss.slot_number, s.id, s.name, s.file_hash, s.file_size,
                    s.notes, s.source_library, s.created_at, s.updated_at,
                    (SELECT COUNT(*) FROM bank_sequence_slots WHERE sequence_id = s.id) as usage_count
             FROM bank_sequence_slots bss
             LEFT JOIN sequences s ON bss.sequence_id = s.id
             WHERE bss.bank_id = ?1
             ORDER BY bss.slot_number",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![bank_id], |row| {
            let slot_number: i32 = row.get(0)?;
            let seq_id: Option<i64> = row.get(1)?;

            if seq_id.is_some() {
                Ok(Some((
                    slot_number,
                    SequenceDto {
                        id: row.get(1)?,
                        name: row.get(2)?,
                        file_hash: row.get(3)?,
                        file_size: row.get(4)?,
                        notes: row.get(5)?,
                        source_library: row.get(6)?,
                        categories: Vec::new(),
                        usage_count: row.get(9)?,
                        created_at: row.get(7)?,
                        updated_at: row.get(8)?,
                    },
                )))
            } else {
                Ok(None)
            }
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        if let Some((slot_number, seq)) = row.map_err(|e| e.to_string())? {
            let idx = (slot_number - 1) as usize;
            if idx < 16 {
                slots[idx].content = Some(seq);
            }
        }
    }

    Ok(slots)
}
