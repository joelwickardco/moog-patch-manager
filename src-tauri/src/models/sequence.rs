use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceDto {
    pub id: i64,
    pub name: String,
    pub file_hash: String,
    pub file_size: i64,
    pub notes: Option<String>,
    pub source_library: Option<String>,
    pub usage_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SequenceFilter {
    pub name_contains: Option<String>,
    pub source_library: Option<String>,
}
