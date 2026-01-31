use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Exported patch data for building library structure
#[derive(Clone)]
pub struct ExportPatch {
    pub name: String,
    pub file_data: Vec<u8>,
}

/// Exported sequence data for building library structure
#[derive(Clone)]
pub struct ExportSequence {
    pub name: String,
    pub file_data: Vec<u8>,
}

/// Bank export configuration
pub struct ExportBank {
    pub bank_number: i32,
    pub name: String,
    pub patches: Vec<Option<ExportPatch>>,      // 16 slots
    pub sequences: Vec<Option<ExportSequence>>, // 16 slots
}

/// Export a library structure to a directory
pub fn export_library_structure(
    banks: Vec<ExportBank>,
    output_dir: &Path,
    library_name: &str,
) -> Result<(), String> {
    // Create root directory with library name
    let library_root = output_dir.join(sanitize_filename(library_name));
    fs::create_dir_all(&library_root)
        .map_err(|e| format!("Failed to create library root directory: {}", e))?;

    // Create library subdirectory inside the library root
    let library_dir = library_root.join("library");
    fs::create_dir_all(&library_dir)
        .map_err(|e| format!("Failed to create library directory: {}", e))?;

    // Create sequences directory
    let sequences_dir = library_dir.join("sequences");
    fs::create_dir_all(&sequences_dir)
        .map_err(|e| format!("Failed to create sequences directory: {}", e))?;

    for bank in banks {
        // Create bank directory
        let bank_dir = library_dir.join(format!("bank{:02}", bank.bank_number));
        fs::create_dir_all(&bank_dir).map_err(|e| {
            format!(
                "Failed to create bank{:02} directory: {}",
                bank.bank_number, e
            )
        })?;

        // Create .bank metadata file
        let bank_file_name = sanitize_filename(&bank.name);
        let bank_file_path = bank_dir.join(format!("{}.bank", bank_file_name));
        File::create(&bank_file_path).map_err(|e| format!("Failed to create .bank file: {}", e))?;

        // Create patch directories
        for (idx, patch_opt) in bank.patches.iter().enumerate() {
            let patch_num = idx + 1;
            let patch_dir = bank_dir.join(format!("patch{:02}", patch_num));
            fs::create_dir_all(&patch_dir)
                .map_err(|e| format!("Failed to create patch directory: {}", e))?;

            if let Some(patch) = patch_opt {
                let patch_file_name = sanitize_filename(&patch.name);
                let patch_file_path = patch_dir.join(format!("{}.mmp", patch_file_name));
                let mut file = File::create(&patch_file_path)
                    .map_err(|e| format!("Failed to create .mmp file: {}", e))?;
                file.write_all(&patch.file_data)
                    .map_err(|e| format!("Failed to write patch data: {}", e))?;
            }
            // If patch is None, leave directory empty (default patch)
        }

        // Create sequence directories
        let seq_bank_dir = sequences_dir.join(format!("bank{:02}", bank.bank_number));
        fs::create_dir_all(&seq_bank_dir)
            .map_err(|e| format!("Failed to create sequence bank directory: {}", e))?;

        for (idx, seq_opt) in bank.sequences.iter().enumerate() {
            let seq_num = idx + 1;
            let seq_dir = seq_bank_dir.join(format!("seq{:02}", seq_num));
            fs::create_dir_all(&seq_dir)
                .map_err(|e| format!("Failed to create sequence directory: {}", e))?;

            if let Some(seq) = seq_opt {
                let seq_file_name = sanitize_filename(&seq.name);
                let seq_file_path = seq_dir.join(format!("{}.mmseq", seq_file_name));
                let mut file = File::create(&seq_file_path)
                    .map_err(|e| format!("Failed to create .mmseq file: {}", e))?;
                file.write_all(&seq.file_data)
                    .map_err(|e| format!("Failed to write sequence data: {}", e))?;
            }
            // If sequence is None, leave directory empty
        }
    }

    Ok(())
}

/// Sanitize a filename by replacing invalid characters
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}
