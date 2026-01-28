use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: Uuid,
    pub document_id: Uuid,
    pub type_name: String,
    pub ordinal: i32,
    pub revision: i32,
    pub payload: Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct BlockDefinition {
    pub type_name: String,
    pub validation_schema: Arc<jsonschema::JSONSchema>, // Compiled schema
    pub complexity_score: u8,
}

pub trait SearchableBlock {
    fn to_search_text(&self, payload: &Value) -> String;
}
