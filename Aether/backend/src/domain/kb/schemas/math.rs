use crate::domain::kb::registry::{BlockSchema, SchemaError};
use serde_json::Value;

pub struct MathSchema;

impl BlockSchema for MathSchema {
    fn validate(&self, payload: &Value) -> Result<(), SchemaError> {
        // Validate math_type (expanded for MATH-01 formal object model)
        let math_type = payload.get("math_type").and_then(|v| v.as_str());
        match math_type {
            Some("theorem")
            | Some("lemma")
            | Some("definition")
            | Some("proposition")
            | Some("corollary")
            | Some("proof")
            | Some("example")
            | Some("problem")
            | Some("note")
            // Legacy aliases
            | Some("axiom") => {}
            Some(other) => {
                return Err(SchemaError::ValidationFailed(format!(
                    "Invalid math_type: {}. Expected one of: theorem, lemma, definition, proposition, corollary, proof, example, problem, note",
                    other
                )))
            }
            None => {
                return Err(SchemaError::ValidationFailed(
                    "Missing 'math_type' field in math block".into(),
                ))
            }
        }

        // Validate latex
        if payload.get("latex").and_then(|v| v.as_str()).is_none() {
            return Err(SchemaError::ValidationFailed(
                "Missing or invalid 'latex' field in math block".into(),
            ));
        }

        // Validate ref_label format if present (MATH-05)
        if let Some(ref_label) = payload.get("ref_label").and_then(|v| v.as_str()) {
            if ref_label.contains(' ') {
                return Err(SchemaError::ValidationFailed(
                    "ref_label must not contain spaces".into(),
                ));
            }
        }

        Ok(())
    }

    fn to_searchable_text(&self, payload: &Value) -> String {
        let latex = payload["latex"].as_str().unwrap_or("");
        let label = payload
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let ref_label = payload
            .get("ref_label")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        // Concatenate for search
        format!("{} {} {}", latex, label, ref_label)
            .trim()
            .to_string()
    }
}
