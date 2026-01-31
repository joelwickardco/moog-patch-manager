use crate::models::{ExportPreview, ExportResult};
use crate::moog::exporter::{export_library_structure, ExportBank, ExportPatch, ExportSequence};
use crate::utils::create_zip;
use crate::AppState;
use rusqlite::params;
use std::path::Path;
use tauri::State;

#[tauri::command]
pub async fn export_library(
    state: State<'_, AppState>,
    library_id: i64,
    output_path: String,
) -> Result<ExportResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    // Get library name
    let library_name: String = conn
        .query_row(
            "SELECT name FROM libraries WHERE id = ?1",
            params![library_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Library not found: {}", e))?;

    // Create temp directory for building structure
    let temp_dir = std::env::temp_dir().join(format!("moog_export_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let mut banks = Vec::new();
    let mut patches_exported = 0;
    let mut sequences_exported = 0;
    let mut empty_slots = 0;

    // Build export banks for this library
    for bank_num in 1..=16 {
        let (bank_id, bank_name): (i64, String) = conn
            .query_row(
                "SELECT id, name FROM banks WHERE library_id = ?1 AND bank_number = ?2",
                params![library_id, bank_num],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| e.to_string())?;

        let mut patches: Vec<Option<ExportPatch>> = vec![None; 16];
        let mut sequences: Vec<Option<ExportSequence>> = vec![None; 16];

        // Get patches for this bank
        let mut patch_stmt = conn
            .prepare(
                "SELECT bps.slot_number, p.name, p.file_data
                 FROM bank_patch_slots bps
                 LEFT JOIN patches p ON bps.patch_id = p.id
                 WHERE bps.bank_id = ?1
                 ORDER BY bps.slot_number",
            )
            .map_err(|e| e.to_string())?;

        let patch_rows = patch_stmt
            .query_map(params![bank_id], |row| {
                let slot_number: i32 = row.get(0)?;
                let name: Option<String> = row.get(1)?;
                let file_data: Option<Vec<u8>> = row.get(2)?;
                Ok((slot_number, name, file_data))
            })
            .map_err(|e| e.to_string())?;

        for row in patch_rows {
            let (slot_number, name, file_data) = row.map_err(|e| e.to_string())?;
            let idx = (slot_number - 1) as usize;
            if idx < 16 {
                if let (Some(name), Some(data)) = (name, file_data) {
                    patches[idx] = Some(ExportPatch {
                        name,
                        file_data: data,
                    });
                    patches_exported += 1;
                } else {
                    empty_slots += 1;
                }
            }
        }

        // Get sequences for this bank
        let mut seq_stmt = conn
            .prepare(
                "SELECT bss.slot_number, s.name, s.file_data
                 FROM bank_sequence_slots bss
                 LEFT JOIN sequences s ON bss.sequence_id = s.id
                 WHERE bss.bank_id = ?1
                 ORDER BY bss.slot_number",
            )
            .map_err(|e| e.to_string())?;

        let seq_rows = seq_stmt
            .query_map(params![bank_id], |row| {
                let slot_number: i32 = row.get(0)?;
                let name: Option<String> = row.get(1)?;
                let file_data: Option<Vec<u8>> = row.get(2)?;
                Ok((slot_number, name, file_data))
            })
            .map_err(|e| e.to_string())?;

        for row in seq_rows {
            let (slot_number, name, file_data) = row.map_err(|e| e.to_string())?;
            let idx = (slot_number - 1) as usize;
            if idx < 16 {
                if let (Some(name), Some(data)) = (name, file_data) {
                    sequences[idx] = Some(ExportSequence {
                        name,
                        file_data: data,
                    });
                    sequences_exported += 1;
                }
            }
        }

        banks.push(ExportBank {
            bank_number: bank_num,
            name: bank_name,
            patches,
            sequences,
        });
    }

    // Build the library structure
    export_library_structure(banks, &temp_dir, &library_name)?;

    // Create ZIP file
    let output = Path::new(&output_path);
    create_zip(&temp_dir, output)?;

    // Get file size
    let file_size = std::fs::metadata(output)
        .map(|m| m.len() as i64)
        .unwrap_or(0);

    // Cleanup temp directory
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(ExportResult {
        library_id,
        library_name,
        output_path,
        file_size,
        banks_exported: 16,
        patches_exported,
        sequences_exported,
        empty_slots,
    })
}

#[tauri::command]
pub async fn preview_export(
    state: State<'_, AppState>,
    library_id: i64,
) -> Result<ExportPreview, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    // Get library name
    let library_name: String = conn
        .query_row(
            "SELECT name FROM libraries WHERE id = ?1",
            params![library_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Library not found: {}", e))?;

    // Count patches in bank_patch_slots for this library's banks
    let total_patches: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM bank_patch_slots bps
             JOIN banks b ON bps.bank_id = b.id
             WHERE b.library_id = ?1 AND bps.patch_id IS NOT NULL",
            params![library_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Count sequences in bank_sequence_slots for this library's banks
    let total_sequences: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM bank_sequence_slots bss
             JOIN banks b ON bss.bank_id = b.id
             WHERE b.library_id = ?1 AND bss.sequence_id IS NOT NULL",
            params![library_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let empty_patch_slots = (16 * 16) - total_patches;
    let empty_sequence_slots = (16 * 16) - total_sequences;

    // Estimate size: sum of all patch and sequence file sizes for this library
    let estimated_size: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(p.file_size), 0) FROM patches p
             JOIN bank_patch_slots bps ON p.id = bps.patch_id
             JOIN banks b ON bps.bank_id = b.id
             WHERE b.library_id = ?1",
            params![library_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let seq_size: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(s.file_size), 0) FROM sequences s
             JOIN bank_sequence_slots bss ON s.id = bss.sequence_id
             JOIN banks b ON bss.bank_id = b.id
             WHERE b.library_id = ?1",
            params![library_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(ExportPreview {
        library_id,
        library_name,
        total_banks: 16,
        total_patches,
        total_sequences,
        empty_patch_slots,
        empty_sequence_slots,
        estimated_size: estimated_size + seq_size + 10000, // Add overhead for structure
    })
}
