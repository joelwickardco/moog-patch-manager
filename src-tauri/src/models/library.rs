use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDto {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub source_filename: Option<String>,
    pub color: Option<String>,
    pub patch_count: i64,
    pub sequence_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryCreate {
    pub name: String,
    pub description: Option<String>,
    pub source_filename: Option<String>,
    pub color: Option<String>,
}
