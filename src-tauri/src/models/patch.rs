use serde::{Deserialize, Serialize};
use super::CategoryDto;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchDto {
    pub id: i64,
    pub library_id: i64,
    pub library_name: String,
    pub name: String,
    pub file_hash: String,
    pub file_size: i64,
    pub is_favorite: bool,
    pub notes: Option<String>,
    pub categories: Vec<CategoryDto>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatchFilter {
    pub library_id: Option<i64>,
    pub is_favorite: Option<bool>,
    pub category_ids: Option<Vec<i64>>,
    pub name_contains: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchCreate {
    pub name: String,
    pub file_data: Vec<u8>,
}
