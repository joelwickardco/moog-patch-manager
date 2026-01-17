use serde::{Deserialize, Serialize};
use super::{PatchDto, SequenceDto};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankDto {
    pub id: i64,
    pub library_id: i64,
    pub bank_number: i32,
    pub name: String,
    pub description: Option<String>,
    pub patches: Vec<Option<PatchDto>>,    // 16 slots
    pub sequences: Vec<Option<SequenceDto>>, // 16 slots
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub library_id: i64,
    pub library_name: String,
    pub patches_imported: i32,
    pub patches_skipped: i32,
    pub sequences_imported: i32,
    pub sequences_skipped: i32,
    pub banks_imported: i32,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ImportResult {
    pub fn new(library_id: i64, library_name: String) -> Self {
        Self {
            library_id,
            library_name,
            patches_imported: 0,
            patches_skipped: 0,
            sequences_imported: 0,
            sequences_skipped: 0,
            banks_imported: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub structure_errors: Vec<String>,
    pub missing_files: Vec<String>,
    pub invalid_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub library_id: i64,
    pub library_name: String,
    pub output_path: String,
    pub file_size: i64,
    pub banks_exported: i32,
    pub patches_exported: i32,
    pub sequences_exported: i32,
    pub empty_slots: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPreview {
    pub library_id: i64,
    pub library_name: String,
    pub total_banks: i32,
    pub total_patches: i32,
    pub total_sequences: i32,
    pub empty_patch_slots: i32,
    pub empty_sequence_slots: i32,
    pub estimated_size: i64,
}
