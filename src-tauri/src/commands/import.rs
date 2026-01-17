use rusqlite::params;
use std::path::Path;
use tauri::State;
use crate::models::{ImportResult, ValidationResult};
use crate::moog::{parse_library, validate_library};
use crate::utils::{calculate_sha256, extract_zip};
use crate::AppState;

#[tauri::command]
pub async fn import_library_zip(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportResult, String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("File not found".to_string());
    }

    // Create temp directory for extraction
    let temp_dir = std::env::temp_dir().join(format!("moog_import_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

    // Extract ZIP
    extract_zip(path, &temp_dir)?;

    // Import from extracted directory
    let result = import_from_directory(&state, &temp_dir).await;

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);

    result
}

#[tauri::command]
pub async fn import_bank_directory(
    state: State<'_, AppState>,
    directory_path: String,
) -> Result<ImportResult, String> {
    let path = Path::new(&directory_path);
    if !path.exists() {
        return Err("Directory not found".to_string());
    }

    import_from_directory(&state, path).await
}

#[tauri::command]
pub async fn validate_library_structure(path: String) -> Result<ValidationResult, String> {
    let path = Path::new(&path);
    if !path.exists() {
        return Err("Path not found".to_string());
    }

    Ok(validate_library(path))
}

async fn import_from_directory(
    state: &State<'_, AppState>,
    path: &Path,
) -> Result<ImportResult, String> {
    let mut result = ImportResult::default();

    // Parse the library structure
    let parsed = parse_library(path)?;
    result.warnings = parsed.warnings;

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    // Import banks
    for bank in &parsed.banks {
        conn.execute(
            "UPDATE banks SET name = ?1, updated_at = CURRENT_TIMESTAMP WHERE bank_number = ?2",
            params![bank.name, bank.bank_number],
        )
        .map_err(|e| e.to_string())?;
        result.banks_imported += 1;
    }

    // Import patches
    for patch in parsed.patches {
        let hash = calculate_sha256(&patch.file_data);

        // Check for duplicate
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM patches WHERE file_hash = ?1",
                params![hash],
                |row| row.get(0),
            )
            .ok();

        let patch_id = if let Some(id) = existing {
            result.patches_skipped += 1;
            id
        } else {
            // Insert new patch
            conn.execute(
                "INSERT INTO patches (name, file_data, file_hash, file_size) VALUES (?1, ?2, ?3, ?4)",
                params![patch.name, patch.file_data, hash, patch.file_data.len() as i64],
            )
            .map_err(|e| e.to_string())?;
            result.patches_imported += 1;
            conn.last_insert_rowid()
        };

        // Link to bank
        let bank_id: i64 = conn
            .query_row(
                "SELECT id FROM banks WHERE bank_number = ?1",
                params![patch.bank_number],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE bank_patches SET patch_id = ?1 WHERE bank_id = ?2 AND patch_number = ?3",
            params![patch_id, bank_id, patch.patch_number],
        )
        .map_err(|e| e.to_string())?;
    }

    // Import sequences
    for seq in parsed.sequences {
        let hash = calculate_sha256(&seq.file_data);

        // Check for duplicate
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM sequences WHERE file_hash = ?1",
                params![hash],
                |row| row.get(0),
            )
            .ok();

        let seq_id = if let Some(id) = existing {
            result.sequences_skipped += 1;
            id
        } else {
            // Insert new sequence
            conn.execute(
                "INSERT INTO sequences (name, file_data, file_hash, file_size) VALUES (?1, ?2, ?3, ?4)",
                params![seq.name, seq.file_data, hash, seq.file_data.len() as i64],
            )
            .map_err(|e| e.to_string())?;
            result.sequences_imported += 1;
            conn.last_insert_rowid()
        };

        // Link to bank
        let bank_id: i64 = conn
            .query_row(
                "SELECT id FROM banks WHERE bank_number = ?1",
                params![seq.bank_number],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE bank_sequences SET sequence_id = ?1 WHERE bank_id = ?2 AND sequence_number = ?3",
            params![seq_id, bank_id, seq.sequence_number],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(result)
}
