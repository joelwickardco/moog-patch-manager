use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagDto {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchDto {
    pub id: i64,
    pub name: String,
    pub file_hash: String,
    pub file_size: i64,
    pub is_favorite: bool,
    pub notes: Option<String>,
    pub source_library: Option<String>,
    pub usage_count: i64,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatchFilter {
    pub is_favorite: Option<bool>,
    pub name_contains: Option<String>,
    pub source_library: Option<String>,
    pub tags: Option<Vec<String>>,
    pub require_all_tags: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchCreate {
    pub name: String,
    pub file_data: Vec<u8>,
}
