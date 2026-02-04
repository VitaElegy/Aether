use crate::domain::kb::registry::{BlockSchema, SchemaError};
use serde_json::Value;

pub struct PaperSchema;

impl BlockSchema for PaperSchema {
    fn validate(&self, payload: &Value) -> Result<(), SchemaError> {
        // Required fields
        if payload.get("title").and_then(|v| v.as_str()).is_none() {
             return Err(SchemaError::ValidationFailed("Missing or invalid 'title' field in paper block".into()));
        }
        
        let ingest_status = payload.get("ingest_status").and_then(|v| v.as_str());
        match ingest_status {
            Some("inbox") | Some("archived") | Some("trash") | Some( "library") => {}, // Added library as valid status
            Some(other) => return Err(SchemaError::ValidationFailed(format!("Invalid ingest_status: {}", other))),
            None => return Err(SchemaError::ValidationFailed("Missing 'ingest_status' field in paper block".into())),
        }

        // Validate Year if present
        if let Some(year) = payload.get("year") {
            if !year.is_u64() {
                 return Err(SchemaError::ValidationFailed("'year' must be an integer".into()));
            }
        }

        Ok(())
    }

    fn to_searchable_text(&self, payload: &Value) -> String {
        let title = payload["title"].as_str().unwrap_or("");
        let abstract_text = payload.get("abstract").and_then(|v| v.as_str()).unwrap_or("");
        let venue = payload.get("venue").and_then(|v| v.as_str()).unwrap_or("");
        let authors = payload.get("authors")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(" "))
            .unwrap_or_default();

        // Concatenate for search
        format!("{} {} {} {}", title, authors, venue, abstract_text).trim().to_string()
    }
}
