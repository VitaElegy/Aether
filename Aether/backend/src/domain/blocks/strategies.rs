use super::models::Block;
use serde_json::Value;

pub fn apply_searchable_trait(block: &mut Block) {
    let mut payload_obj = match block.payload.as_object() {
        Some(obj) => obj.clone(),
        None => return, // Should not happen if schema valid
    };

    let text_mirror = extract_text(&block.type_name, &block.payload);
    
    // Upsert text_mirror field
    payload_obj.insert("text_mirror".to_string(), Value::String(text_mirror));
    block.payload = Value::Object(payload_obj);
}

fn extract_text(type_name: &str, payload: &Value) -> String {
    match type_name {
        // Standard Types
        "paragraph" => payload["markdown"].as_str().unwrap_or("").to_string(),
        "heading" => payload["text"].as_str().unwrap_or("").to_string(),
        "math" => payload["latex"].as_str().unwrap_or("").to_string(),
        "code" => payload["code"].as_str().unwrap_or("").to_string(),
        
        // Math KB Types
        "axiom" => {
            let label = payload["label"].as_str().unwrap_or("");
            let content = payload["content"].as_str().unwrap_or("");
            format!("{} {}", label, content).trim().to_string()
        },
        "theorem" => {
            let label = payload["label"].as_str().unwrap_or("");
            let content = payload["content"].as_str().unwrap_or("");
            format!("{} {}", label, content).trim().to_string()
        },
        "definition" => {
            let term = payload["term"].as_str().unwrap_or("");
            let content = payload["content"].as_str().unwrap_or("");
            format!("{} {}", term, content).trim().to_string()
        },
        "proof" => {
            // Proofs are just steps
            payload["steps"].as_str().unwrap_or("").to_string()
        },
        
        // Fallback: Try to find common text fields
        _ => {
            if let Some(t) = payload.get("text") {
                t.as_str().unwrap_or("").to_string()
            } else if let Some(c) = payload.get("content") {
                c.as_str().unwrap_or("").to_string()
            } else {
                "".to_string()
            }
        }
    }
}

pub fn extract_references(type_name: &str, payload: &Value) -> Vec<uuid::Uuid> {
    let mut refs = Vec::new();
    
    // 1. Explicit Fields
    match type_name {
        "theorem" => {
            if let Some(pid) = payload.get("proof_id").and_then(|v| v.as_str()) {
                if let Ok(id) = uuid::Uuid::parse_str(pid) {
                    refs.push(id);
                }
            }
        },
        "proof" => {
            if let Some(tid) = payload.get("theorem_id").and_then(|v| v.as_str()) {
                if let Ok(id) = uuid::Uuid::parse_str(tid) {
                    refs.push(id);
                }
            }
        },
        _ => {}
    }

    // 2. Content Scanning (Regex for [[uuid]])
    // For now, simple strict UUID scanning in string fields
    // Implement stricter regex if needed
    
    refs
}
