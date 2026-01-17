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

    let mut result = ImportResult::new(library_id, unique_name);
    result.warnings = parsed.warnings;

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

        // Check for duplicate across ALL libraries
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
            // Insert new patch with library_id
            conn.execute(
                "INSERT INTO patches (library_id, name, file_data, file_hash, file_size) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![library_id, patch.name, patch.file_data, hash, patch.file_data.len() as i64],
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

        // Check for duplicate across ALL libraries
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
            // Insert new sequence with library_id
            conn.execute(
                "INSERT INTO sequences (library_id, name, file_data, file_hash, file_size) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![library_id, seq.name, seq.file_data, hash, seq.file_data.len() as i64],
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

    // Update library counts
    conn.execute(
        "UPDATE libraries SET patch_count = ?1, sequence_count = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
        params![result.patches_imported, result.sequences_imported, library_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(result)
}
