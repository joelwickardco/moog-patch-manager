use rusqlite::params;
use std::path::Path;
use tauri::State;
use crate::models::{ImportResult, ValidationResult};
use crate::moog::{parse_library, validate_library};
use crate::utils::{calculate_sha256, extract_zip};
use crate::AppState;

/// Extract library name from a ZIP filename
/// e.g., "Moog Factory Sounds v2.zip" -> "Moog Factory Sounds v2"
fn extract_library_name(file_path: &Path) -> String {
    file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Imported Library".to_string())
}

/// Generate a unique library name if the proposed name already exists
fn generate_unique_library_name(conn: &rusqlite::Connection, base_name: &str) -> Result<String, String> {
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM libraries WHERE name = ?1)",
            params![base_name],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if !exists {
        return Ok(base_name.to_string());
    }

    // Try adding numeric suffix
    for i in 2..100 {
        let candidate = format!("{} ({})", base_name, i);
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM libraries WHERE name = ?1)",
                params![candidate],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        if !exists {
            return Ok(candidate);
        }
    }

    Err("Could not generate unique library name".to_string())
}

#[tauri::command]
pub async fn import_library_zip(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportResult, String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("File not found".to_string());
    }

    // Extract library name from ZIP filename
    let library_name = extract_library_name(path);
    let source_filename = path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    // Create temp directory for extraction
    let temp_dir = std::env::temp_dir().join(format!("moog_import_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

    // Extract ZIP
    extract_zip(path, &temp_dir)?;

    // Import from extracted directory with library name
    let result = import_from_directory(&state, &temp_dir, &library_name, source_filename).await;

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

    // Use directory name as library name
    let library_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Imported Library".to_string());

    import_from_directory(&state, path, &library_name, None).await
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
    library_name: &str,
    source_filename: Option<String>,
) -> Result<ImportResult, String> {
    // Parse the library structure
    let parsed = parse_library(path)?;

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();

    // Generate unique library name if needed
    let unique_name = generate_unique_library_name(conn, library_name)?;

    // Create the library entry
    conn.execute(
        "INSERT INTO libraries (name, source_filename) VALUES (?1, ?2)",
        params![unique_name, source_filename],
    )
    .map_err(|e| e.to_string())?;

    let library_id = conn.last_insert_rowid();

    let mut result = ImportResult::new(library_id, unique_name.clone());
    result.warnings = parsed.warnings;

    // Create 16 banks for this library
    for bank_num in 1..=16 {
        // Check if we have a parsed bank name for this number
        let bank_name = parsed.banks.iter()
            .find(|b| b.bank_number == bank_num)
            .map(|b| b.name.clone())
            .unwrap_or_else(|| format!("Bank {:02}", bank_num));

        conn.execute(
            "INSERT INTO banks (library_id, bank_number, name) VALUES (?1, ?2, ?3)",
            params![library_id, bank_num, bank_name],
        )
        .map_err(|e| e.to_string())?;

        let bank_id = conn.last_insert_rowid();

        // Create 16 patch slots for this bank (initially empty)
        for slot_num in 1..=16 {
            conn.execute(
                "INSERT INTO bank_patch_slots (bank_id, slot_number, patch_id) VALUES (?1, ?2, NULL)",
                params![bank_id, slot_num],
            )
            .map_err(|e| e.to_string())?;
        }

        // Create 16 sequence slots for this bank (initially empty)
        for slot_num in 1..=16 {
            conn.execute(
                "INSERT INTO bank_sequence_slots (bank_id, slot_number, sequence_id) VALUES (?1, ?2, NULL)",
                params![bank_id, slot_num],
            )
            .map_err(|e| e.to_string())?;
        }

        result.banks_created += 1;
    }

    // Import patches
    for patch in parsed.patches {
        let hash = calculate_sha256(&patch.file_data);

        // Check for duplicate in global content store
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM patches WHERE file_hash = ?1",
                params![hash],
                |row| row.get(0),
            )
            .ok();

        let patch_id = if let Some(id) = existing {
            result.patches_reused += 1;
            id
        } else {
            // Insert new patch into global content store with source_library
            conn.execute(
                "INSERT INTO patches (name, file_data, file_hash, file_size, source_library) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![patch.name, patch.file_data, hash, patch.file_data.len() as i64, unique_name],
            )
            .map_err(|e| e.to_string())?;
            result.patches_imported += 1;
            conn.last_insert_rowid()
        };

        // Link to bank slot
        let bank_id: i64 = conn
            .query_row(
                "SELECT id FROM banks WHERE library_id = ?1 AND bank_number = ?2",
                params![library_id, patch.bank_number],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE bank_patch_slots SET patch_id = ?1 WHERE bank_id = ?2 AND slot_number = ?3",
            params![patch_id, bank_id, patch.patch_number],
        )
        .map_err(|e| e.to_string())?;

        result.slots_populated += 1;
    }

    // Import sequences
    for seq in parsed.sequences {
        let hash = calculate_sha256(&seq.file_data);

        // Check for duplicate in global content store
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM sequences WHERE file_hash = ?1",
                params![hash],
                |row| row.get(0),
            )
            .ok();

        let seq_id = if let Some(id) = existing {
            result.sequences_reused += 1;
            id
        } else {
            // Insert new sequence into global content store with source_library
            conn.execute(
                "INSERT INTO sequences (name, file_data, file_hash, file_size, source_library) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![seq.name, seq.file_data, hash, seq.file_data.len() as i64, unique_name],
            )
            .map_err(|e| e.to_string())?;
            result.sequences_imported += 1;
            conn.last_insert_rowid()
        };

        // Link to bank slot
        let bank_id: i64 = conn
            .query_row(
                "SELECT id FROM banks WHERE library_id = ?1 AND bank_number = ?2",
                params![library_id, seq.bank_number],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE bank_sequence_slots SET sequence_id = ?1 WHERE bank_id = ?2 AND slot_number = ?3",
            params![seq_id, bank_id, seq.sequence_number],
        )
        .map_err(|e| e.to_string())?;

        result.slots_populated += 1;
    }

    // Update library timestamp
    conn.execute(
        "UPDATE libraries SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        params![library_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(result)
}
