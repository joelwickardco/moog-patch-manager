use std::fs;
use std::path::{Path, PathBuf};

/// Parsed patch data from filesystem
pub struct ParsedPatch {
    pub name: String,
    pub file_data: Vec<u8>,
    pub bank_number: i32,
    pub patch_number: i32,
}

/// Parsed sequence data from filesystem
pub struct ParsedSequence {
    pub name: String,
    pub file_data: Vec<u8>,
    pub bank_number: i32,
    pub sequence_number: i32,
}

/// Parsed bank data from filesystem
pub struct ParsedBank {
    pub bank_number: i32,
    pub name: String,
}

/// Result of parsing a library directory
pub struct ParsedLibrary {
    pub banks: Vec<ParsedBank>,
    pub patches: Vec<ParsedPatch>,
    pub sequences: Vec<ParsedSequence>,
    pub warnings: Vec<String>,
}

/// Find the library/ directory, handling both direct and nested ZIP structures
/// Supports:
///   - Direct: extracted_dir/library/
///   - Nested: extracted_dir/SomeFolder/library/
fn find_library_dir(base_path: &Path) -> Result<PathBuf, String> {
    // First, check if library/ exists directly
    let direct_library = base_path.join("library");
    if direct_library.exists() && direct_library.is_dir() {
        return Ok(direct_library);
    }

    // Otherwise, look for a single subdirectory containing library/
    if let Ok(entries) = fs::read_dir(base_path) {
        let subdirs: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let path = e.path();
                path.is_dir() && !e.file_name().to_string_lossy().starts_with('.')
            })
            .collect();

        // Check each subdirectory for a library/ folder
        for subdir in &subdirs {
            let nested_library = subdir.path().join("library");
            if nested_library.exists() && nested_library.is_dir() {
                return Ok(nested_library);
            }
        }
    }

    Err("Missing 'library/' directory. Expected either 'library/' at root or inside a single folder.".to_string())
}

/// Parse a Moog library directory structure
pub fn parse_library(library_path: &Path) -> Result<ParsedLibrary, String> {
    let mut result = ParsedLibrary {
        banks: Vec::new(),
        patches: Vec::new(),
        sequences: Vec::new(),
        warnings: Vec::new(),
    };

    // Check for library/ root directory - handle both direct and nested structures
    let library_dir = find_library_dir(library_path)?;

    // Parse all 16 banks
    for bank_num in 1..=16 {
        let bank_dir = library_dir.join(format!("bank{:02}", bank_num));
        if !bank_dir.exists() {
            result.warnings.push(format!("Missing bank{:02} directory", bank_num));
            continue;
        }

        // Find .bank file for bank name
        let bank_name = find_bank_name(&bank_dir, bank_num, &mut result.warnings);
        result.banks.push(ParsedBank {
            bank_number: bank_num,
            name: bank_name,
        });

        // Parse patches in this bank
        for patch_num in 1..=16 {
            let patch_dir = bank_dir.join(format!("patch{:02}", patch_num));
            if !patch_dir.exists() {
                result
                    .warnings
                    .push(format!("Missing patch{:02} in bank{:02}", patch_num, bank_num));
                continue;
            }

            if let Some(patch) = parse_patch_dir(&patch_dir, bank_num, patch_num) {
                result.patches.push(patch);
            }
        }
    }

    // Parse sequences
    let sequences_dir = library_dir.join("sequences");
    if sequences_dir.exists() {
        for bank_num in 1..=16 {
            let seq_bank_dir = sequences_dir.join(format!("bank{:02}", bank_num));
            if !seq_bank_dir.exists() {
                continue;
            }

            for seq_num in 1..=16 {
                let seq_dir = seq_bank_dir.join(format!("seq{:02}", seq_num));
                if !seq_dir.exists() {
                    continue;
                }

                if let Some(seq) = parse_sequence_dir(&seq_dir, bank_num, seq_num) {
                    result.sequences.push(seq);
                }
            }
        }
    }

    Ok(result)
}

fn find_bank_name(bank_dir: &Path, bank_num: i32, warnings: &mut Vec<String>) -> String {
    let default_name = format!("Bank {:02}", bank_num);

    if let Ok(entries) = fs::read_dir(bank_dir) {
        let bank_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "bank")
                    .unwrap_or(false)
            })
            .collect();

        if bank_files.is_empty() {
            warnings.push(format!("No .bank file in bank{:02}, using default name", bank_num));
            return default_name;
        }

        if bank_files.len() > 1 {
            warnings.push(format!(
                "Multiple .bank files in bank{:02}, using first",
                bank_num
            ));
        }

        bank_files[0]
            .path()
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or(default_name)
    } else {
        default_name
    }
}

fn parse_patch_dir(patch_dir: &Path, bank_num: i32, patch_num: i32) -> Option<ParsedPatch> {
    if let Ok(entries) = fs::read_dir(patch_dir) {
        let mmp_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "mmp")
                    .unwrap_or(false)
            })
            .collect();

        if mmp_files.is_empty() {
            return None; // Empty patch slot (uses default)
        }

        let mmp_path = mmp_files[0].path();
        let name = mmp_path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())?;

        let file_data = fs::read(&mmp_path).ok()?;

        Some(ParsedPatch {
            name,
            file_data,
            bank_number: bank_num,
            patch_number: patch_num,
        })
    } else {
        None
    }
}

fn parse_sequence_dir(seq_dir: &Path, bank_num: i32, seq_num: i32) -> Option<ParsedSequence> {
    if let Ok(entries) = fs::read_dir(seq_dir) {
        let mmseq_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "mmseq")
                    .unwrap_or(false)
            })
            .collect();

        if mmseq_files.is_empty() {
            return None;
        }

        let seq_path = mmseq_files[0].path();
        let name = seq_path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())?;

        let file_data = fs::read(&seq_path).ok()?;

        Some(ParsedSequence {
            name,
            file_data,
            bank_number: bank_num,
            sequence_number: seq_num,
        })
    } else {
        None
    }
}
