use std::fs;
use std::path::Path;
use crate::models::ValidationResult;

/// Validate a Moog library directory structure
pub fn validate_library(path: &Path) -> ValidationResult {
    let mut result = ValidationResult {
        is_valid: true,
        structure_errors: Vec::new(),
        missing_files: Vec::new(),
        invalid_files: Vec::new(),
    };

    // Check for library/ root
    let library_dir = path.join("library");
    if !library_dir.exists() {
        result.is_valid = false;
        result
            .structure_errors
            .push("Missing 'library/' root directory".to_string());
        return result;
    }

    // Check for 16 banks
    for bank_num in 1..=16 {
        let bank_dir = library_dir.join(format!("bank{:02}", bank_num));
        if !bank_dir.exists() {
            result
                .missing_files
                .push(format!("library/bank{:02}/", bank_num));
            continue;
        }

        // Check for .bank file
        let has_bank_file = fs::read_dir(&bank_dir)
            .map(|entries| {
                entries.filter_map(|e| e.ok()).any(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "bank")
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        if !has_bank_file {
            result
                .missing_files
                .push(format!("library/bank{:02}/<name>.bank", bank_num));
        }

        // Check for 16 patch folders
        for patch_num in 1..=16 {
            let patch_dir = bank_dir.join(format!("patch{:02}", patch_num));
            if !patch_dir.exists() {
                result.missing_files.push(format!(
                    "library/bank{:02}/patch{:02}/",
                    bank_num, patch_num
                ));
            } else {
                // Validate any .mmp files present
                if let Ok(entries) = fs::read_dir(&patch_dir) {
                    let mmp_files: Vec<_> = entries
                        .filter_map(|e| e.ok())
                        .filter(|e| {
                            e.path()
                                .extension()
                                .map(|ext| ext == "mmp")
                                .unwrap_or(false)
                        })
                        .collect();

                    if mmp_files.len() > 1 {
                        result.invalid_files.push(format!(
                            "Multiple .mmp files in bank{:02}/patch{:02}/",
                            bank_num, patch_num
                        ));
                    }
                }
            }
        }
    }

    // Check sequences structure (optional but validate if exists)
    let sequences_dir = library_dir.join("sequences");
    if sequences_dir.exists() {
        for bank_num in 1..=16 {
            let seq_bank_dir = sequences_dir.join(format!("bank{:02}", bank_num));
            if !seq_bank_dir.exists() {
                continue; // Sequences are optional
            }

            for seq_num in 1..=16 {
                let seq_dir = seq_bank_dir.join(format!("seq{:02}", seq_num));
                if seq_dir.exists() {
                    // Validate any .mmseq files
                    if let Ok(entries) = fs::read_dir(&seq_dir) {
                        let mmseq_files: Vec<_> = entries
                            .filter_map(|e| e.ok())
                            .filter(|e| {
                                e.path()
                                    .extension()
                                    .map(|ext| ext == "mmseq")
                                    .unwrap_or(false)
                            })
                            .collect();

                        if mmseq_files.len() > 1 {
                            result.invalid_files.push(format!(
                                "Multiple .mmseq files in sequences/bank{:02}/seq{:02}/",
                                bank_num, seq_num
                            ));
                        }
                    }
                }
            }
        }
    }

    // Set invalid if there are structure errors
    if !result.structure_errors.is_empty() {
        result.is_valid = false;
    }

    result
}
