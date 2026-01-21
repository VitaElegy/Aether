use crate::domain::kb::registry::{BlockSchema, SchemaError};
use serde_json::Value;

pub struct MarkdownSchema;

impl BlockSchema for MarkdownSchema {
    fn validate(&self, payload: &Value) -> Result<(), SchemaError> {
        if payload.get("content").and_then(|v| v.as_str()).is_none() {
            return Err(SchemaError::ValidationFailed("Missing or invalid 'content' field in markdown block".into()));
        }
        Ok(())
    }

    fn to_searchable_text(&self, payload: &Value) -> String {
        payload["content"].as_str().unwrap_or("").to_string()
    }
}
