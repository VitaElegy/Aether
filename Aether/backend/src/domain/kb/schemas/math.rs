use crate::domain::kb::registry::{BlockSchema, SchemaError};
use serde_json::Value;

pub struct MathSchema;

impl BlockSchema for MathSchema {
    fn validate(&self, payload: &Value) -> Result<(), SchemaError> {
        // Validate math_type
        let math_type = payload.get("math_type").and_then(|v| v.as_str());
        match math_type {
            Some("theorem") | Some("axiom") | Some("proof") | Some("definition") => {},
            Some(other) => return Err(SchemaError::ValidationFailed(format!("Invalid math_type: {}", other))),
            None => return Err(SchemaError::ValidationFailed("Missing 'math_type' field in math block".into())),
        }

        // Validate latex
        if payload.get("latex").and_then(|v| v.as_str()).is_none() {
             return Err(SchemaError::ValidationFailed("Missing or invalid 'latex' field in math block".into()));
        }
        
        Ok(())
    }

    fn to_searchable_text(&self, payload: &Value) -> String {
        let latex = payload["latex"].as_str().unwrap_or("");
        let label = payload.get("label").and_then(|v| v.as_str()).unwrap_or("");
        // Concatenate latex and label for search
        format!("{} {}", latex, label).trim().to_string()
    }
}
